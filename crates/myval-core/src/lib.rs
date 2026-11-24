pub mod auth;
pub mod cache;
pub mod career;
pub mod engine;
pub mod error;
pub mod glz;
pub mod live;
pub mod loadout;
pub mod models;
pub mod pd;
pub mod rate_limit;
pub mod valorant_api;

pub use engine::MyValEngineCore;
pub use error::MyValError;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::auth::{parse_redirect_url, region_to_shard};
    use crate::cache::LocalCache;
    use crate::error::MyValError;
    use crate::live::{detect_party_groups, format_display_name, visible_account_level};
    use crate::loadout::RawPlayerLoadout;
    use crate::models::assets::{CompetitiveTier, MapInfo, Skin, SkinLevel};
    use crate::models::mmr::{PlayerMmrSummary, RankInfo};
    use crate::models::store::Wallet;
    use crate::pd::{enrich_storefront, RawStorefrontResponse};

    #[test]
    fn test_parse_redirect_url_fragment_and_errors() {
        let ok_url = "https://playvalorant.com/opt_in#access_token=abc.def.ghi&scope=openid&iss=https%3A%2F%2Fauth.riotgames.com&id_token=id123&token_type=Bearer&expires_in=3600";
        let parsed = parse_redirect_url(ok_url).expect("should parse fragment tokens");
        assert_eq!(parsed.access_token, "abc.def.ghi");
        assert_eq!(parsed.id_token, "id123");
        assert_eq!(parsed.expires_in_secs, 3600);

        let err_url = "https://playvalorant.com/opt_in#error=access_denied&error_description=User%20canceled";
        let err = parse_redirect_url(err_url).unwrap_err();
        assert!(matches!(err, MyValError::Unauthorized(_)));
    }

    #[test]
    fn test_region_to_shard_mapping() {
        assert_eq!(region_to_shard("latam").shard, "na");
        assert_eq!(region_to_shard("BR").shard, "na");
        assert_eq!(region_to_shard("ap").shard, "ap");
        assert_eq!(region_to_shard("eu").shard, "eu");
        assert_eq!(region_to_shard("kr").shard, "kr");
        assert_eq!(region_to_shard("na").shard, "na");
    }

    #[test]
    fn test_storefront_v3_deserialization_and_level_uuid_enrichment() {
        let cache = LocalCache::open(":memory:").unwrap();
        let skin = Skin {
            uuid: "skin-kuronami".to_string(),
            display_name: "Kuronami Vandal".to_string(),
            weapon_uuid: "wep-vandal".to_string(),
            weapon_name: "Vandal".to_string(),
            content_tier_uuid: Some("tier-excl".to_string()),
            content_tier_name: Some("Exclusive".to_string()),
            content_tier_color: "#F5955B".to_string(),
            display_icon: Some("https://cdn.example/kuronami.png".to_string()),
            wallpaper: None,
            chromas: vec![],
            levels: vec![SkinLevel {
                uuid: "level-kuronami-1".to_string(),
                display_name: "Kuronami Vandal".to_string(),
                display_icon: None,
                streamed_video: Some("https://cdn.example/kuronami.mp4".to_string()),
            }],
            is_wishlisted: false,
        };
        cache.replace_category("skin", &[skin.clone()], |s| s.uuid.clone()).unwrap();
        cache.rebuild_skin_level_index(&[skin]).unwrap();
        cache.toggle_wishlist("skin-kuronami").unwrap();

        let raw_json = r#"{
            "SkinsPanelLayout": {
                "SingleItemOffers": ["level-kuronami-1"],
                "SingleItemStoreOffers": [
                    {
                        "OfferID": "level-kuronami-1",
                        "Cost": { "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741": 2375 },
                        "Rewards": [
                            {
                                "ItemTypeID": "e7c63390-eda7-46e0-bb7a-a6abdacd2433",
                                "ItemID": "level-kuronami-1"
                            }
                        ]
                    }
                ],
                "SingleItemOffersRemainingDurationInSeconds": 43210
            },
            "BonusStore": {
                "BonusStoreRemainingDurationInSeconds": 99999,
                "BonusStoreOffers": [
                    {
                        "BonusOfferID": "nm-offer-1",
                        "Offer": {
                            "OfferID": "level-kuronami-1",
                            "Cost": { "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741": 2375 },
                            "Rewards": [{ "ItemTypeID": "e7c63390-eda7-46e0-bb7a-a6abdacd2433", "ItemID": "level-kuronami-1" }]
                        },
                        "DiscountPercent": 40,
                        "DiscountCosts": { "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741": 1425 },
                        "IsSeen": false
                    }
                ]
            }
        }"#;
        let raw: RawStorefrontResponse = serde_json::from_str(raw_json).unwrap();
        let snap = enrich_storefront(
            &cache,
            &raw,
            Wallet {
                vp: 3000,
                radianite: 50,
                kingdom_credits: 4500,
            },
        );
        assert_eq!(snap.daily_remaining_secs, 43210);
        assert_eq!(snap.daily_offers.len(), 1);
        assert_eq!(snap.daily_offers[0].skin_name, "Kuronami Vandal");
        assert_eq!(snap.daily_offers[0].weapon_name, "Vandal");
        assert_eq!(snap.daily_offers[0].cost_vp, 2375);
        assert!(snap.daily_offers[0].is_wishlisted);
        let nm = snap.night_market.expect("night market present");
        assert_eq!(nm.offers[0].discount_price_vp, 1425);
        assert_eq!(nm.offers[0].discount_percent, 40);
    }

    #[test]
    fn test_party_stack_detection_duos_trios_and_cross_team_isolation() {
        use crate::career::build_cached_match_party_map;

        let players = vec![
            ("b1".to_string(), "Blue".to_string()),
            ("b2".to_string(), "Blue".to_string()),
            ("b3".to_string(), "Blue".to_string()),
            ("b4".to_string(), "Blue".to_string()),
            ("r1".to_string(), "Red".to_string()),
            ("r2".to_string(), "Red".to_string()),
        ];
        let mut hist = HashMap::new();
        hist.insert("b1".into(), vec!["curr".into(), "m1".into(), "m2".into()]);
        hist.insert("b2".into(), vec!["curr".into(), "m0".into(), "m1".into(), "m2".into()]);
        hist.insert("b3".into(), vec!["curr".into(), "m0".into(), "m9".into()]);
        hist.insert("b4".into(), vec!["curr".into(), "x1".into(), "x2".into()]);
        hist.insert("r1".into(), vec!["curr".into(), "m1".into(), "m2".into(), "m8".into(), "m7".into()]);
        hist.insert("r2".into(), vec!["curr".into(), "m8".into(), "m7".into()]);

        let mut match_parties = HashMap::new();
        match_parties.insert(
            "m0".into(),
            build_cached_match_party_map(
                "competitive",
                [("b2", "party-b23"), ("b3", "party-b23")],
            ),
        );
        match_parties.insert(
            "m1".into(),
            build_cached_match_party_map(
                "competitive",
                [("b1", "party-b12"), ("b2", "party-b12"), ("r1", "party-r1-solo")],
            ),
        );
        match_parties.insert(
            "m2".into(),
            build_cached_match_party_map(
                "competitive",
                [("b1", "party-b12"), ("b2", "party-b12"), ("r1", "party-r1-solo")],
            ),
        );
        match_parties.insert(
            "m8".into(),
            build_cached_match_party_map(
                "competitive",
                [("r1", "party-r12"), ("r2", "party-r12")],
            ),
        );
        match_parties.insert(
            "m7".into(),
            build_cached_match_party_map(
                "competitive",
                [("r1", "party-r12"), ("r2", "party-r12")],
            ),
        );

        let groups = detect_party_groups(&players, &hist, &match_parties, None, Some("curr"));
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].label, "TRIO A");
        assert_eq!(groups[0].team_id, "Blue");
        assert_eq!(groups[0].member_puuids, vec!["b1", "b2", "b3"]);
        assert_eq!(groups[1].label, "DUO B");
        assert_eq!(groups[1].team_id, "Red");
        assert_eq!(groups[1].member_puuids, vec!["r1", "r2"]);
    }

    #[test]
    fn test_party_detection_blocks_false_solo_requeue_nil_uuid_deathmatch_and_respects_live_party() {
        use crate::cache::CachedMatchPartyMap;
        use crate::career::{build_cached_match_party_map, is_valid_riot_party_id};

        assert!(!is_valid_riot_party_id(""));
        assert!(!is_valid_riot_party_id("00000000-0000-0000-0000-000000000000"));
        assert!(is_valid_riot_party_id("A1B2C3D4-1111-2222-3333-444455556666"));

        // Case A: Solos sharing the exact previous match (e.g., opponents or random teammates who requeued).
        let players_a = vec![
            ("me".to_string(), "Blue".to_string()),
            ("harry".to_string(), "Blue".to_string()),
            ("err".to_string(), "Red".to_string()),
            ("madara".to_string(), "Red".to_string()),
        ];
        let mut hist_a = HashMap::new();
        hist_a.insert("me".into(), vec!["curr".into(), "last_match".into(), "m_a".into()]);
        hist_a.insert("harry".into(), vec!["curr".into(), "last_match".into(), "m_b".into()]);
        hist_a.insert("err".into(), vec!["curr".into(), "enemy_last".into(), "m_c".into()]);
        hist_a.insert("madara".into(), vec!["curr".into(), "enemy_last".into(), "m_d".into()]);

        let mut parties_a = HashMap::new();
        parties_a.insert(
            "last_match".into(),
            build_cached_match_party_map(
                "competitive",
                [("me", "party-solo-me"), ("harry", "party-solo-harry")],
            ),
        );
        parties_a.insert(
            "enemy_last".into(),
            build_cached_match_party_map(
                "competitive",
                [("err", "party-solo-err"), ("madara", "party-solo-madara")],
            ),
        );

        assert!(
            detect_party_groups(&players_a, &hist_a, &parties_a, None, Some("curr")).is_empty(),
            "solo players sharing a previous match with different partyIds must never form a DUO"
        );

        // Case B: Missing match detail, nil UUIDs, deathmatch queue, or single non-latest shared match.
        let players_b = vec![
            ("p1".to_string(), "Blue".to_string()),
            ("p2".to_string(), "Blue".to_string()),
            ("p3".to_string(), "Red".to_string()),
            ("p4".to_string(), "Red".to_string()),
        ];
        let mut hist_b = HashMap::new();
        hist_b.insert(
            "p1".into(),
            vec!["m_missing".into(), "m_nil1".into(), "m_nil2".into(), "m_dm".into(), "m_old".into()],
        );
        hist_b.insert(
            "p2".into(),
            vec!["m_missing".into(), "m_nil1".into(), "m_nil2".into(), "m_dm".into()],
        );
        hist_b.insert("p3".into(), vec!["m_old".into()]);
        hist_b.insert("p4".into(), vec!["m_solo_latest".into(), "m_old".into()]);

        let mut parties_b = HashMap::new();
        // Bypass builder for m_nil1/m_nil2/m_dm to verify detect_party_groups guards directly too.
        parties_b.insert(
            "m_nil1".into(),
            CachedMatchPartyMap {
                queue_id: "competitive".into(),
                player_parties: HashMap::from([
                    ("p1".into(), "00000000-0000-0000-0000-000000000000".into()),
                    ("p2".into(), "00000000-0000-0000-0000-000000000000".into()),
                ]),
            },
        );
        parties_b.insert(
            "m_nil2".into(),
            CachedMatchPartyMap {
                queue_id: "competitive".into(),
                player_parties: HashMap::from([
                    ("p1".into(), "00000000-0000-0000-0000-000000000000".into()),
                    ("p2".into(), "00000000-0000-0000-0000-000000000000".into()),
                ]),
            },
        );
        parties_b.insert(
            "m_dm".into(),
            CachedMatchPartyMap {
                queue_id: "deathmatch".into(),
                player_parties: HashMap::from([
                    ("p1".into(), "party-dm-1".into()),
                    ("p2".into(), "party-dm-1".into()),
                ]),
            },
        );
        parties_b.insert(
            "m_old".into(),
            build_cached_match_party_map(
                "competitive",
                [("p3", "party-old"), ("p4", "party-old")],
            ),
        );

        assert!(detect_party_groups(&players_b, &hist_b, &parties_b, None, None).is_empty());

        // Case C1: Live GLZ solo roster overrides 2 historical shared party matches for `me`,
        // while still detecting unrelated teammates' verified duo.
        let players_c = vec![
            ("me".to_string(), "Blue".to_string()),
            ("former_duo".to_string(), "Blue".to_string()),
            ("other1".to_string(), "Blue".to_string()),
            ("other2".to_string(), "Blue".to_string()),
        ];
        let mut hist_c = HashMap::new();
        hist_c.insert("me".into(), vec!["m1".into(), "m2".into()]);
        hist_c.insert("former_duo".into(), vec!["m1".into(), "m2".into()]);
        hist_c.insert("other1".into(), vec!["m3".into(), "m4".into()]);
        hist_c.insert("other2".into(), vec!["m3".into(), "m4".into()]);
        let mut parties_c = HashMap::new();
        for mid in ["m1", "m2"] {
            parties_c.insert(
                mid.into(),
                build_cached_match_party_map(
                    "competitive",
                    [("me", "party-old-me"), ("former_duo", "party-old-me")],
                ),
            );
        }
        for mid in ["m3", "m4"] {
            parties_c.insert(
                mid.into(),
                build_cached_match_party_map(
                    "competitive",
                    [("other1", "party-others"), ("other2", "party-others")],
                ),
            );
        }
        let solo_live = vec!["me".to_string()];
        let groups_c1 = detect_party_groups(
            &players_c,
            &hist_c,
            &parties_c,
            Some(("me", &solo_live)),
            None,
        );
        assert_eq!(groups_c1.len(), 1);
        assert_eq!(groups_c1[0].label, "DUO A");
        assert_eq!(groups_c1[0].member_puuids, vec!["other1", "other2"]);

        // Case C2: Live GLZ premade roster groups `me` + `friend` with 0 shared history,
        // and still respects enemy team isolation.
        let players_c2 = vec![
            ("ME".to_string(), "Blue".to_string()),
            ("friend".to_string(), "Blue".to_string()),
            ("enemy_friend".to_string(), "Red".to_string()),
        ];
        let live_duo = vec![
            "me".to_string(),
            "friend".to_string(),
            "enemy_friend".to_string(),
        ];
        let groups_c2 = detect_party_groups(
            &players_c2,
            &HashMap::new(),
            &HashMap::new(),
            Some(("me", &live_duo)),
            None,
        );
        assert_eq!(groups_c2.len(), 1);
        assert_eq!(groups_c2[0].label, "DUO A");
        assert_eq!(groups_c2[0].member_puuids, vec!["ME", "friend"]);
    }

    #[test]
    fn test_match_party_cache_roundtrip_builder_and_schema_v2() {
        use crate::career::build_cached_match_party_map;

        // 1. Verify builder normalizes casing, drops nil UUIDs, and clears DM/custom party entries.
        let comp_map = build_cached_match_party_map(
            "Competitive",
            [
                ("Player-A", "Party-1"),
                ("player-b", "00000000-0000-0000-0000-000000000000"),
                ("", "party-empty-sub"),
            ],
        );
        assert_eq!(comp_map.queue_id, "competitive");
        assert_eq!(comp_map.player_parties.len(), 1);
        assert_eq!(
            comp_map.player_parties.get("player-a").map(String::as_str),
            Some("party-1")
        );

        let dm_map = build_cached_match_party_map("deathmatch", [("Player-A", "Party-1")]);
        assert!(dm_map.player_parties.is_empty());

        let custom_map = build_cached_match_party_map("Custom", [("Player-A", "Party-1")]);
        assert!(custom_map.player_parties.is_empty());

        // 2. Verify SQLite match_party_cache roundtrip and schema v2 metadata stamp.
        let cache = LocalCache::open(":memory:").expect("open in-memory cache");
        assert!(cache.get_cached_match_party_map("match-xyz").is_none());
        cache
            .put_cached_match_party_map("match-xyz", &comp_map)
            .expect("write cached match party map");
        let loaded = cache
            .get_cached_match_party_map("match-xyz")
            .expect("read cached match party map");
        assert_eq!(loaded.queue_id, comp_map.queue_id);
        assert_eq!(loaded.player_parties, comp_map.player_parties);

        // 3. Verify candidate collection skips cross-team coincidences and prioritizes latest shared matches.
        let players = vec![
            ("b1".to_string(), "Blue".to_string()),
            ("b2".to_string(), "Blue".to_string()),
            ("r1".to_string(), "Red".to_string()),
        ];
        let mut hist = HashMap::new();
        hist.insert(
            "b1".into(),
            vec![
                "curr".into(),
                "m_latest".into(),
                "m_cross".into(),
                "m_dup".into(),
                "m_dup".into(),
            ],
        );
        hist.insert("b2".into(), vec!["curr".into(), "m_latest".into()]);
        hist.insert("r1".into(), vec!["curr".into(), "m_cross".into()]);
        let candidates = crate::live::collect_shared_candidate_match_ids(
            &players,
            &hist,
            Some("curr"),
            10,
        );
        assert_eq!(candidates, vec!["m_latest".to_string()]);
    }

    #[test]
    fn test_resolve_recent_outcomes_strictly_competitive_only() {
        use crate::live::resolve_recent_outcomes;
        use crate::models::career::{
            CareerSnapshot, CompetitiveMatchUpdate, MatchHistoryItem, PlayerProfileSnapshot,
        };
        use crate::models::mmr::PlayerMmrSummary;

        let cache = LocalCache::open(":memory:").expect("open in-memory cache");
        let empty_mmr = PlayerMmrSummary::default();

        // 1. Career fallback for `is_me`: when rr_trend is empty and recent_matches contains only
        //    non-competitive queues (deathmatch, swiftplay, unrated, custom), dots must stay empty.
        let mixed_career = CareerSnapshot {
            puuid: "me-puuid".into(),
            rr_trend: Vec::new(),
            recent_matches: vec![
                MatchHistoryItem {
                    queue_id: "deathmatch".into(),
                    won: false,
                    my_team_rounds: 35,
                    enemy_team_rounds: 40,
                    ..Default::default()
                },
                MatchHistoryItem {
                    queue_id: "swiftplay".into(),
                    won: false,
                    my_team_rounds: 2,
                    enemy_team_rounds: 5,
                    ..Default::default()
                },
                MatchHistoryItem {
                    queue_id: "unrated".into(),
                    won: true,
                    my_team_rounds: 13,
                    enemy_team_rounds: 8,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        cache.put_cached_career("me-puuid", &mixed_career).unwrap();
        assert!(resolve_recent_outcomes(&cache, "me-puuid", true, &empty_mmr).is_empty());

        // 2. If career.recent_matches contains mixed queues + competitive, ONLY competitive entries
        //    are included (draw when rounds tie, win/loss otherwise).
        let mixed_with_comp = CareerSnapshot {
            puuid: "me-comp".into(),
            rr_trend: Vec::new(),
            recent_matches: vec![
                MatchHistoryItem {
                    queue_id: "deathmatch".into(),
                    won: false,
                    my_team_rounds: 20,
                    enemy_team_rounds: 40,
                    ..Default::default()
                },
                MatchHistoryItem {
                    queue_id: "competitive".into(),
                    won: true,
                    my_team_rounds: 13,
                    enemy_team_rounds: 10,
                    ..Default::default()
                },
                MatchHistoryItem {
                    queue_id: "unrated".into(),
                    won: false,
                    my_team_rounds: 5,
                    enemy_team_rounds: 13,
                    ..Default::default()
                },
                MatchHistoryItem {
                    queue_id: "Competitive".into(),
                    won: false,
                    my_team_rounds: 14,
                    enemy_team_rounds: 14,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        cache.put_cached_career("me-comp", &mixed_with_comp).unwrap();
        assert_eq!(
            resolve_recent_outcomes(&cache, "me-comp", true, &empty_mmr),
            vec![1, 0]
        );

        // 3. Other-player profile fallback: prefers prof.rr_trend (strictly competitive),
        //    and ignores non-competitive recent_matches.
        let other_prof = PlayerProfileSnapshot {
            puuid: "other-puuid".into(),
            rr_trend: vec![
                CompetitiveMatchUpdate {
                    rr_delta: 21,
                    ..Default::default()
                },
                CompetitiveMatchUpdate {
                    rr_delta: -16,
                    ..Default::default()
                },
            ],
            recent_matches: vec![MatchHistoryItem {
                queue_id: "deathmatch".into(),
                won: false,
                my_team_rounds: 10,
                enemy_team_rounds: 40,
                ..Default::default()
            }],
            ..Default::default()
        };
        cache.put_cached_player_profile("other-puuid", &other_prof).unwrap();
        assert_eq!(
            resolve_recent_outcomes(&cache, "other-puuid", false, &empty_mmr),
            vec![1, -1]
        );
    }

    #[test]
    fn test_weapon_kill_stream_icon_and_match_detail_backfill() {
        use crate::models::assets::Weapon;
        use crate::models::career::{
            MatchDetailSnapshot, RoundKillEvent, RoundReplayData, ScoreboardPlayer,
        };

        // Weapon without explicit kill_stream_icon derives /killstreamicon.png from display_icon
        let ghost = Weapon {
            uuid: "wep-ghost".into(),
            display_name: "Ghost".into(),
            category: "Sidearm".into(),
            display_icon: Some("https://media.valorant-api.com/weapons/wep-ghost/displayicon.png".into()),
            kill_stream_icon: None,
            ..Default::default()
        };
        assert_eq!(
            ghost.kill_stream_icon_url().as_deref(),
            Some("https://media.valorant-api.com/weapons/wep-ghost/killstreamicon.png")
        );

        let vandal = Weapon {
            uuid: "wep-vandal".into(),
            display_name: "Vandal".into(),
            category: "Rifle".into(),
            kill_stream_icon: Some("https://cdn.example/vandal-killstream.png".into()),
            ..Default::default()
        };

        let cache = LocalCache::open(":memory:").unwrap();
        cache
            .replace_category("weapon", &[ghost, vandal], |w| w.uuid.clone())
            .unwrap();

        let legacy_detail = MatchDetailSnapshot {
            match_id: "m-kill-icons".into(),
            my_team_id: "Blue".into(),
            scoreboard: vec![ScoreboardPlayer {
                puuid: "p-me".into(),
                display_name: "Me#001".into(),
                agent_name: "Jett".into(),
                team_id: "Blue".into(),
                ..Default::default()
            }],
            rounds: vec![RoundReplayData {
                round_num: 1,
                kills: vec![RoundKillEvent {
                    weapon_name: "Ghost".into(),
                    weapon_icon: None,
                    player_positions: vec![Default::default()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        cache.put_cached_match_detail(&legacy_detail).unwrap();

        let loaded = cache
            .get_cached_match_detail("m-kill-icons", "p-me")
            .expect("cached match detail should backfill missing kill icons");
        assert_eq!(
            loaded.rounds[0].kills[0].weapon_icon.as_deref(),
            Some("https://media.valorant-api.com/weapons/wep-ghost/killstreamicon.png")
        );
    }

    #[test]
    fn test_real_display_name_resolution_and_hide_account_level() {
        let mut names = HashMap::new();
        names.insert("p-secret".into(), ("HiddenPro".into(), "0001".into()));

        // Real Riot ID is always shown (case-insensitive PUUID lookup) even when streamer/incognito mode is on.
        assert_eq!(
            format_display_name("P-SECRET", &names, true, false, "Jett"),
            "HiddenPro#0001"
        );
        assert_eq!(
            format_display_name("p-secret", &names, true, true, "Jett"),
            "HiddenPro#0001"
        );
        // Unknown player falls back cleanly without ever showing "Selecting..." as a name.
        assert_eq!(
            format_display_name("p-missing", &names, false, false, "Selecting agent..."),
            "Player"
        );
        assert_eq!(
            format_display_name("p-missing", &names, false, false, "Deadlock"),
            "Deadlock"
        );
        assert_eq!(visible_account_level(350, true, false), None);
        assert_eq!(visible_account_level(350, true, true), Some(350));

        // Verify name-service null fields never crash deserialization of the whole batch.
        let raw_names = r#"[
            {"DisplayName": "SamplePlayer#0001", "Subject": "u1", "GameName": "SamplePlayer", "TagLine": "0001"},
            {"DisplayName": "Fallback#777", "Subject": "u2", "GameName": null, "TagLine": null}
        ]"#;
        let parsed_names: Vec<crate::pd::RawNameEntry> = serde_json::from_str(raw_names).unwrap();
        assert_eq!(parsed_names.len(), 2);
        assert_eq!(parsed_names[0].game_name, "SamplePlayer");
        assert_eq!(parsed_names[1].display_name, "Fallback#777");
        assert_eq!(parsed_names[1].game_name, "");
        assert_eq!(crate::pd::fallback_name_service_shard("ap"), "na");
        assert_eq!(crate::pd::fallback_name_service_shard("eu"), "na");
        assert_eq!(crate::pd::fallback_name_service_shard("na"), "eu");
    }

    #[test]
    fn test_party_state_machine_and_action_guards() {
        use crate::glz::{RawMatchmakingData, RawPartyResp};
        use crate::live::{
            build_party_snapshot_from_parts, validate_party_change_queue,
            validate_party_generate_invite, validate_party_join_queue, validate_party_leave_queue,
            validate_party_set_accessibility, validate_party_set_ready,
        };
        use crate::models::live::PartyMemberInfo;

        let make_member = |puuid: &str, is_me: bool, is_owner: bool, is_ready: bool| PartyMemberInfo {
            puuid: puuid.into(),
            display_name: puuid.into(),
            is_ready,
            is_owner,
            is_me,
            ..Default::default()
        };

        // 1. In matchmaking queue: queue switching, ready toggle, privacy toggle, invite gen, and start queue MUST be blocked!
        let in_queue_snap = build_party_snapshot_from_parts(
            RawPartyResp {
                id: "party-1".into(),
                state: "MATCHMAKING".into(),
                accessibility: "OPEN".into(),
                eligible_queues: vec!["competitive".into(), "unrated".into()],
                matchmaking_data: Some(RawMatchmakingData {
                    queue_id: "competitive".into(),
                }),
                ..Default::default()
            },
            vec![make_member("me", true, true, true)],
        );
        assert!(in_queue_snap.is_in_queue);
        assert!(!in_queue_snap.can_change_queue);
        assert!(!in_queue_snap.can_toggle_ready);
        assert!(!in_queue_snap.can_toggle_accessibility);
        assert!(!in_queue_snap.can_generate_invite_code);
        assert!(!in_queue_snap.can_start_queue);
        assert!(in_queue_snap.can_leave_queue);
        assert!(validate_party_change_queue(&in_queue_snap, "unrated").is_err());
        assert!(validate_party_set_ready(&in_queue_snap).is_err());
        assert!(validate_party_set_accessibility(&in_queue_snap).is_err());
        assert!(validate_party_generate_invite(&in_queue_snap).is_err());
        assert!(validate_party_join_queue(&in_queue_snap).is_err());
        assert!(validate_party_leave_queue(&in_queue_snap).is_ok());

        // 2. Non-owner in idle lobby: cannot change queue, start queue, or toggle privacy, but CAN toggle ready.
        let non_owner_snap = build_party_snapshot_from_parts(
            RawPartyResp {
                id: "party-2".into(),
                state: "DEFAULT".into(),
                accessibility: "CLOSED".into(),
                eligible_queues: vec!["competitive".into(), "unrated".into()],
                matchmaking_data: Some(RawMatchmakingData {
                    queue_id: "competitive".into(),
                }),
                ..Default::default()
            },
            vec![
                make_member("leader", false, true, true),
                make_member("me", true, false, false),
            ],
        );
        assert!(!non_owner_snap.can_change_queue);
        assert!(!non_owner_snap.can_start_queue);
        assert!(!non_owner_snap.can_toggle_accessibility);
        assert!(non_owner_snap.can_toggle_ready);
        assert!(validate_party_change_queue(&non_owner_snap, "unrated").is_err());
        assert!(validate_party_set_ready(&non_owner_snap).is_ok());

        // 3. Leader in idle lobby with unready teammate & ineligible deathmatch:
        // can switch to eligible "unrated", CANNOT switch to already-active "competitive" or ineligible "deathmatch", CANNOT start queue until all ready!
        let leader_unready = build_party_snapshot_from_parts(
            RawPartyResp {
                id: "party-3".into(),
                state: "DEFAULT".into(),
                accessibility: "CLOSED".into(),
                eligible_queues: vec!["competitive".into(), "unrated".into(), "swiftplay".into()],
                matchmaking_data: Some(RawMatchmakingData {
                    queue_id: "competitive".into(),
                }),
                ..Default::default()
            },
            vec![
                make_member("me", true, true, true),
                make_member("friend", false, false, false),
            ],
        );
        assert!(leader_unready.can_change_queue);
        assert!(!leader_unready.can_start_queue);
        assert!(!leader_unready.can_generate_invite_code); // CLOSED party
        assert!(validate_party_change_queue(&leader_unready, "unrated").is_ok());
        assert!(validate_party_change_queue(&leader_unready, "competitive").is_err());
        assert!(validate_party_change_queue(&leader_unready, "deathmatch").is_err());
        assert!(validate_party_join_queue(&leader_unready).is_err());
        assert!(validate_party_generate_invite(&leader_unready).is_err());

        // 4. Match found (MATCHMADE_GAME_STARTING): cannot leave queue either!
        let starting_snap = build_party_snapshot_from_parts(
            RawPartyResp {
                id: "party-4".into(),
                state: "MATCHMADE_GAME_STARTING".into(),
                accessibility: "OPEN".into(),
                ..Default::default()
            },
            vec![make_member("me", true, true, true)],
        );
        assert!(starting_snap.is_match_starting);
        assert!(!starting_snap.can_leave_queue);
        assert!(!starting_snap.can_change_queue);
        assert!(validate_party_leave_queue(&starting_snap).is_err());
    }

    #[test]
    fn test_pregame_deserialization_and_owned_skin_level_guards() {
        use std::collections::HashSet;
        use crate::glz::RawPreGameMatchResp;
        use crate::loadout::{apply_skin_to_raw_loadout, randomize_raw_loadout, RawLoadoutGun};

        let raw_pg = r#"{
            "ID": "match-pg-1",
            "MapID": "/Game/Maps/Corrode/Corrode",
            "QueueID": "competitive",
            "PregameState": "character_select_active",
            "PhaseTimeRemainingNS": 36500000000,
            "EnemyTeamSize": 5,
            "EnemyTeamLockCount": 3,
            "IsRanked": true,
            "AllyTeam": {
                "TeamID": "Red",
                "Players": []
            }
        }"#;
        let parsed: RawPreGameMatchResp = serde_json::from_str(raw_pg).unwrap();
        assert_eq!(parsed.queue_id, "competitive");
        assert_eq!(parsed.phase_time_remaining_ns / 1_000_000_000, 36);
        assert_eq!(parsed.ally_team.unwrap().team_id, "Red");

        // Verify skin ownership & partial upgrade level resolution (never equip unowned Level 4!)
        let cache = LocalCache::open(":memory:").unwrap();
        let vandal_skin = Skin {
            uuid: "skin-vandal-1".into(),
            display_name: "Kuronami Vandal".into(),
            weapon_uuid: "wep-vandal".into(),
            weapon_name: "Vandal".into(),
            content_tier_uuid: None,
            content_tier_name: Some("Exclusive".into()),
            content_tier_color: "#F5955B".into(),
            display_icon: None,
            wallpaper: None,
            chromas: vec![],
            levels: vec![
                SkinLevel { uuid: "lvl-1".into(), display_name: "L1".into(), display_icon: None, streamed_video: None },
                SkinLevel { uuid: "lvl-2".into(), display_name: "L2".into(), display_icon: None, streamed_video: None },
                SkinLevel { uuid: "lvl-4".into(), display_name: "L4".into(), display_icon: None, streamed_video: None },
            ],
            is_wishlisted: false,
        };
        cache.replace_category("skin", &[vandal_skin.clone()], |s| s.uuid.clone()).unwrap();
        cache.rebuild_skin_level_index(&[vandal_skin]).unwrap();

        let mut loadout = RawPlayerLoadout {
            subject: "p1".into(),
            guns: vec![RawLoadoutGun {
                id: "wep-vandal".into(),
                ..Default::default()
            }],
            ..Default::default()
        };

        // Unowned skin MUST be rejected when owned set is known
        let other_owned = HashSet::from(["skin-other".to_string()]);
        assert!(apply_skin_to_raw_loadout(&cache, &mut loadout, "wep-vandal", "skin-vandal-1", &other_owned, &HashSet::new(), None, &HashSet::new(), None).is_err());

        // Owned up to Level 2: MUST select lvl-2, NOT unowned lvl-4!
        let owned_skins = HashSet::from(["skin-vandal-1".to_string()]);
        let owned_levels = HashSet::from(["lvl-1".to_string(), "lvl-2".to_string()]);
        apply_skin_to_raw_loadout(&cache, &mut loadout, "wep-vandal", "skin-vandal-1", &owned_skins, &owned_levels, None, &HashSet::new(), None).unwrap();
        assert_eq!(loadout.guns[0].skin_level_id, "lvl-2");

        // Randomizing with zero owned skins MUST fail cleanly instead of equipping unowned paid skins!
        assert!(randomize_raw_loadout(&cache, &mut loadout, &HashSet::new(), &HashSet::new(), 42).is_err());
    }

    #[test]
    fn test_sqlite_cache_roundtrip_assets_mmr_ttl_and_wishlist() {
        let cache = LocalCache::open(":memory:").unwrap();
        let tier = CompetitiveTier {
            tier: 22,
            tier_name: "ASCENDANT 2".to_string(),
            division_name: "ASCENDANT".to_string(),
            color: "#1B8F6B".to_string(),
            background_color: "#0D2420".to_string(),
            large_icon: Some("https://cdn.example/asc2.png".to_string()),
        };
        cache
            .replace_category("competitivetier", &[tier], |t| t.tier.to_string())
            .unwrap();
        assert_eq!(
            cache.lookup_competitive_tier(22).unwrap().tier_name,
            "ASCENDANT 2"
        );

        let mmr = PlayerMmrSummary {
            puuid: "p-1".into(),
            current_rank: RankInfo {
                tier: 22,
                tier_name: "ASCENDANT 2".into(),
                division_name: "ASCENDANT".into(),
                rr: 64,
                icon_url: None,
                color_hex: "#1B8F6B".into(),
            },
            peak_rank: RankInfo::default(),
            peak_rank_act_name: String::new(),
            previous_act_rank: RankInfo::default(),
            leaderboard_rank: None,
            recent_results: vec![true, false],
            recent_outcomes: vec![1, -1],
            recent_results_loaded: true,
        };
        cache.put_cached_mmr(&mmr).unwrap();
        assert_eq!(cache.get_cached_mmr("p-1", 600).unwrap().current_rank.rr, 64);
        assert!(cache.get_cached_mmr("p-1", -1).is_none());

        assert!(cache.toggle_wishlist("SKIN-1").unwrap());
        assert_eq!(cache.get_wishlist_uuids().unwrap(), vec!["skin-1"]);
        assert!(!cache.toggle_wishlist("skin-1").unwrap());
        assert!(cache.get_wishlist_uuids().unwrap().is_empty());
    }

    #[test]
    fn test_null_safe_player_loadout_deserialization() {
        let raw_json = r#"{
            "Subject": "puuid-123",
            "Version": 25,
            "Guns": [
                {
                    "ID": "9c82e19d-4575-0200-1a81-3eacf00cf872",
                    "CharmInstanceID": null,
                    "CharmID": null,
                    "CharmLevelID": null,
                    "SkinID": "skin-1",
                    "SkinLevelID": "level-1",
                    "ChromaID": "chroma-1",
                    "Attachments": null
                }
            ],
            "Sprays": [
                {
                    "EquipSlotID": "slot-1",
                    "SprayID": "spray-1",
                    "SprayLevelID": null
                }
            ],
            "Identity": {
                "PlayerCardID": "card-1",
                "PlayerTitleID": null,
                "AccountLevel": 152,
                "PreferredLevelBorderID": null,
                "HideAccountLevel": false
            },
            "Incognito": false
        }"#;
        let parsed: RawPlayerLoadout =
            serde_json::from_str(raw_json).expect("must deserialize null fields safely");
        assert_eq!(parsed.subject, "puuid-123");
        assert_eq!(parsed.guns.len(), 1);
        assert_eq!(parsed.identity.preferred_level_border_id, "");
    }

    #[test]
    fn test_minimap_coordinate_projection() {
        use crate::career::transform_minimap_coords;

        let map = MapInfo {
            display_name: "Ascent".into(),
            x_multiplier: 0.00007,
            y_multiplier: -0.00007,
            x_scalar_to_add: 0.813895,
            y_scalar_to_add: 0.573242,
            ..Default::default()
        };
        let (nx, ny) = transform_minimap_coords(1200.0, -3500.0, &map);
        assert!((0.03..=0.97).contains(&nx));
        assert!((0.03..=0.97).contains(&ny));
    }

    #[test]
    fn test_loadout_preset_crud() {
        let cache = LocalCache::open(":memory:").unwrap();
        let preset = cache
            .save_loadout_preset("puuid-test", "Kuronami Purple", "#A78BFA", 10, &["Kuronami Vandal".into()], "{}")
            .unwrap();
        assert_eq!(cache.list_loadout_presets(Some("puuid-test")).unwrap().len(), 1);
        assert_eq!(cache.list_loadout_presets(Some("other-puuid")).unwrap().len(), 0);
        cache.delete_loadout_preset(&preset.preset_id).unwrap();
        assert!(cache.list_loadout_presets(Some("puuid-test")).unwrap().is_empty());
    }

    #[test]
    fn test_deterministic_previous_act_and_peak_rank() {
        use crate::models::assets::{CompetitiveTier, SeasonInfo};
        use crate::pd::{
            summarize_mmr, RawCompetitiveUpdate, RawMmrResponse, RawQueueSkill, RawSeasonInfo,
        };

        let cache = LocalCache::open(":memory:").unwrap();
        let tiers = vec![
            CompetitiveTier {
                tier: 15,
                tier_name: "PLATINUM 1".into(),
                division_name: "PLATINUM".into(),
                ..Default::default()
            },
            CompetitiveTier {
                tier: 20,
                tier_name: "DIAMOND 3".into(),
                division_name: "DIAMOND".into(),
                ..Default::default()
            },
            CompetitiveTier {
                tier: 21,
                tier_name: "ASCENDANT 1".into(),
                division_name: "ASCENDANT".into(),
                ..Default::default()
            },
            CompetitiveTier {
                tier: 25,
                tier_name: "IMMORTAL 2".into(),
                division_name: "IMMORTAL".into(),
                ..Default::default()
            },
        ];
        cache.replace_category("competitivetier", &tiers, |t| t.tier.to_string()).unwrap();

        let seasons = vec![
            SeasonInfo {
                uuid: "act-old".into(),
                display_name: "ACT IV".into(),
                start_time: "2024-01-01T00:00:00Z".into(),
                end_time: "2024-03-01T00:00:00Z".into(),
                ..Default::default()
            },
            SeasonInfo {
                uuid: "act-prev".into(),
                display_name: "ACT V".into(),
                start_time: "2025-08-01T00:00:00Z".into(),
                end_time: "2025-10-01T00:00:00Z".into(),
                ..Default::default()
            },
            SeasonInfo {
                uuid: "act-curr".into(),
                display_name: "ACT VI".into(),
                start_time: (chrono::Utc::now() - chrono::Duration::days(15)).format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                end_time: (chrono::Utc::now() + chrono::Duration::days(45)).format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                ..Default::default()
            },
        ];
        cache.replace_category("season", &seasons, |s| s.uuid.clone()).unwrap();

        let mut seasonal = HashMap::new();
        seasonal.insert(
            "act-old".into(),
            RawSeasonInfo {
                season_id: "act-old".into(),
                competitive_tier: 15,
                ..Default::default()
            },
        );
        seasonal.insert(
            "act-prev".into(),
            RawSeasonInfo {
                season_id: "act-prev".into(),
                competitive_tier: 21,
                wins_by_tier: Some(HashMap::from([("25".to_string(), 3)])),
                ..Default::default()
            },
        );
        seasonal.insert(
            "act-curr".into(),
            RawSeasonInfo {
                season_id: "act-curr".into(),
                competitive_tier: 20,
                ..Default::default()
            },
        );

        let mut skills = HashMap::new();
        skills.insert(
            "competitive".into(),
            RawQueueSkill {
                seasonal_info_by_season_id: Some(seasonal),
            },
        );

        let raw = RawMmrResponse {
            latest_competitive_update: Some(RawCompetitiveUpdate {
                tier_after_update: 20,
                ranked_rating_after_update: 40,
                ..Default::default()
            }),
            queue_skills: skills,
        };

        let summary = summarize_mmr(&cache, "p-me", &raw);
        assert_eq!(summary.current_rank.tier, 20);
        assert_eq!(summary.peak_rank.tier, 25);
        assert_eq!(summary.previous_act_rank.tier, 25);

        // Also verify when previous act has only competitive_tier without higher wins_by_tier,
        // older acts (Platinum 1) never override the chronological previous Act (Ascendant 1).
        if let Some(comp) = raw
            .queue_skills
            .get("competitive")
            .and_then(|q| q.seasonal_info_by_season_id.as_ref())
        {
            let mut updated_seasonal = comp.clone();
            if let Some(prev) = updated_seasonal.get_mut("act-prev") {
                prev.wins_by_tier = None;
            }
            let raw2 = RawMmrResponse {
                latest_competitive_update: raw.latest_competitive_update,
                queue_skills: HashMap::from([(
                    "competitive".into(),
                    RawQueueSkill {
                        seasonal_info_by_season_id: Some(updated_seasonal),
                    },
                )]),
            };
            let summary2 = summarize_mmr(&cache, "p-me", &raw2);
            assert_eq!(summary2.previous_act_rank.tier, 21);
        }
    }

    #[test]
    fn test_featured_live_skins_filter_and_unbounded_extraction() {
        use std::collections::HashMap;
        use crate::glz::{
            RawCoreGameLoadoutEntry, RawCoreLoadoutDetail, RawCoreLoadoutItem,
            RawCoreLoadoutSocket, RawSocketItem,
        };
        use crate::live::{
            extract_skins_for_subject, is_featured_live_skin, TIER_EXCLUSIVE_UUID,
            TIER_PREMIUM_UUID, TIER_ULTRA_UUID,
        };
        use crate::models::assets::SkinChroma;

        let make_skin = |id: &str, name: &str, weapon: &str, tier_uuid: Option<&str>, tier_name: Option<&str>, levels: usize| -> Skin {
            Skin {
                uuid: id.into(),
                display_name: name.into(),
                weapon_uuid: format!("wep-{weapon}"),
                weapon_name: weapon.into(),
                content_tier_uuid: tier_uuid.map(str::to_string),
                content_tier_name: tier_name.map(str::to_string),
                content_tier_color: "#D1548D".into(),
                display_icon: Some(format!("https://cdn.example/{id}.png")),
                wallpaper: None,
                chromas: vec![SkinChroma {
                    uuid: format!("{id}-chroma-2"),
                    display_name: "Black Variant".into(),
                    display_icon: Some(format!("https://cdn.example/{id}-black.png")),
                    swatch: None,
                    streamed_video: None,
                }],
                levels: (0..levels)
                    .map(|i| SkinLevel {
                        uuid: format!("{id}-lvl-{i}"),
                        display_name: format!("Level {}", i + 1),
                        display_icon: None,
                        streamed_video: None,
                    })
                    .collect(),
                is_wishlisted: false,
            }
        };

        // 8 featured skins (Premium, Exclusive/Limited, Ultra, + animated Deluxe) + 2 basic skins
        let skins = vec![
            make_skin("s1", "Kuronami Vandal", "Vandal", Some(TIER_EXCLUSIVE_UUID), Some("Exclusive Edition"), 4),
            make_skin("s2", "Reaver Phantom", "Phantom", Some(TIER_PREMIUM_UUID), Some("Premium Edition"), 4),
            make_skin("s3", "Elderflame Operator", "Operator", Some(TIER_ULTRA_UUID), Some("Ultra Edition"), 4),
            make_skin("s4", "Arcane Sheriff", "Sheriff", Some(TIER_EXCLUSIVE_UUID), Some("Exclusive Edition"), 2),
            make_skin("s5", "Sovereign Ghost", "Ghost", Some(TIER_PREMIUM_UUID), Some("Premium Edition"), 4),
            make_skin("s6", "Champions 2021 Karambit", "Melee", Some(TIER_EXCLUSIVE_UUID), Some("Exclusive Edition"), 2),
            make_skin("s7", "VCT x SEN Classic", "Classic", Some(TIER_EXCLUSIVE_UUID), Some("Exclusive Edition"), 3),
            make_skin("s8", "Combat Crafts Bulldog", "Bulldog", Some("0cebb8be-46d7-c12a-d306-e9907bfc5a25"), Some("Deluxe Edition"), 2),
            make_skin("s9", "Sakura Stinger", "Stinger", Some("0cebb8be-46d7-c12a-d306-e9907bfc5a25"), Some("Deluxe Edition"), 1),
            make_skin("s10", "Standard Odin", "Odin", None, None, 1),
        ];

        assert!(is_featured_live_skin(&skins[0])); // Exclusive
        assert!(is_featured_live_skin(&skins[1])); // Premium
        assert!(is_featured_live_skin(&skins[2])); // Ultra
        assert!(is_featured_live_skin(&skins[7])); // Animated Deluxe (levels > 1)
        assert!(!is_featured_live_skin(&skins[8])); // Non-animated Deluxe excluded
        assert!(!is_featured_live_skin(&skins[9])); // Standard excluded

        let cache = LocalCache::open(":memory:").unwrap();
        cache.replace_category("skin", &skins, |s| s.uuid.clone()).unwrap();
        cache.rebuild_skin_level_index(&skins).unwrap();

        let mut items = HashMap::new();
        for (idx, s) in skins.iter().enumerate() {
            let mut sockets = HashMap::new();
            sockets.insert(
                "bcef87d6-209b-46c6-8b19-fbe40bd95abc".to_string(),
                RawCoreLoadoutSocket {
                    id: "bcef87d6-209b-46c6-8b19-fbe40bd95abc".to_string(),
                    item: RawSocketItem {
                        id: s.uuid.clone(),
                        type_id: String::new(),
                    },
                },
            );
            if idx == 0 {
                sockets.insert(
                    "3ad1b2b2-acdb-4524-852f-954a77ddae05".to_string(),
                    RawCoreLoadoutSocket {
                        id: "3ad1b2b2-acdb-4524-852f-954a77ddae05".to_string(),
                        item: RawSocketItem {
                            id: "s1-chroma-2".to_string(),
                            type_id: String::new(),
                        },
                    },
                );
            }
            items.insert(
                format!("slot-{idx}"),
                RawCoreLoadoutItem {
                    id: s.weapon_uuid.clone(),
                    type_id: String::new(),
                    sockets,
                },
            );
        }

        let entry = RawCoreGameLoadoutEntry {
            subject: String::new(),
            character_id: "char-1".into(),
            loadout: RawCoreLoadoutDetail {
                subject: "p-1".into(),
                items,
                ..Default::default()
            },
            items: HashMap::new(),
        };
        let extracted = extract_skins_for_subject(&cache, Some(&entry));
        // Must return ALL 8 featured/animated skins (never truncated at 6!) and exclude basic skins.
        assert_eq!(extracted.len(), 8);
        assert_eq!(extracted[0].skin_name, "Kuronami Vandal");
        assert_eq!(extracted[0].skin_icon.as_deref(), Some("https://cdn.example/s1-black.png"));
        assert_eq!(extracted[0].tier_name.as_deref(), Some("Exclusive"));
        assert_eq!(extracted[7].skin_name, "Combat Crafts Bulldog");
        assert_eq!(extracted[7].tier_name.as_deref(), Some("Animated"));
    }

    #[test]
    fn test_coregame_and_pregame_loadouts_null_safety_subject_and_fallback() {
        use crate::glz::{RawCoreGameLoadoutsResp, RawPreGameLoadoutsResp};
        use crate::live::{
            extract_skins_for_subject, extract_skins_from_items, resolve_coregame_loadout_for_player,
            TIER_EXCLUSIVE_UUID,
        };
        use crate::models::assets::{SkinChroma, SkinLevel};

        let cache = LocalCache::open(":memory:").unwrap();
        let skins = vec![
            Skin {
                uuid: "skin-kuro".into(),
                display_name: "Kuronami Vandal".into(),
                weapon_uuid: "wep-vandal".into(),
                weapon_name: "Vandal".into(),
                content_tier_uuid: Some(TIER_EXCLUSIVE_UUID.into()),
                content_tier_name: Some("Exclusive Edition".into()),
                content_tier_color: "#F5955B".into(),
                display_icon: Some("https://cdn.example/kuro-base.png".into()),
                wallpaper: None,
                chromas: vec![SkinChroma {
                    uuid: "chroma-kuro-purple".into(),
                    display_name: "Purple".into(),
                    display_icon: Some("https://cdn.example/kuro-purple.png".into()),
                    swatch: None,
                    streamed_video: None,
                }],
                levels: vec![
                    SkinLevel { uuid: "lvl-1".into(), display_name: "L1".into(), display_icon: None, streamed_video: None },
                    SkinLevel { uuid: "lvl-2".into(), display_name: "L2".into(), display_icon: None, streamed_video: None },
                ],
                is_wishlisted: false,
            },
            Skin {
                uuid: "skin-sakura".into(),
                display_name: "Sakura Vandal".into(),
                weapon_uuid: "wep-vandal".into(),
                weapon_name: "Vandal".into(),
                content_tier_uuid: Some("0cebb8be-46d7-c12a-d306-e9907bfc5a25".into()),
                content_tier_name: Some("Deluxe Edition".into()),
                content_tier_color: "#009587".into(),
                display_icon: Some("https://cdn.example/sakura.png".into()),
                wallpaper: None,
                chromas: vec![],
                levels: vec![SkinLevel { uuid: "sak-1".into(), display_name: "L1".into(), display_icon: None, streamed_video: None }],
                is_wishlisted: false,
            },
        ];
        cache.replace_category("skin", &skins, |s| s.uuid.clone()).unwrap();
        cache.rebuild_skin_level_index(&skins).unwrap();

        // Core-Game loadouts with top-level Subject, uppercase Socket UUIDs, a null Sockets entry, and a Battlepass-only player
        let raw_cg_json = r#"{
            "Loadouts": [
                {
                    "Subject": "P-TOP-LEVEL",
                    "CharacterID": "agent-jett",
                    "Loadout": {
                        "Subject": null,
                        "Items": {
                            "wep-vandal": {
                                "ID": "wep-vandal",
                                "Sockets": {
                                    "BCEF87D6-209B-46C6-8B19-FBE40BD95ABC": {
                                        "ID": "BCEF87D6-209B-46C6-8B19-FBE40BD95ABC",
                                        "Item": { "ID": "skin-kuro", "TypeID": "e7c63390-eda7-46e0-bb7a-a6abdacd2433" }
                                    },
                                    "3AD1B2B2-ACDB-4524-852F-954A77DDAE05": {
                                        "ID": "3AD1B2B2-ACDB-4524-852F-954A77DDAE05",
                                        "Item": { "ID": "chroma-kuro-purple", "TypeID": "3ad1b2b2-acdb-4524-852f-954a77ddae05" }
                                    }
                                }
                            },
                            "wep-empty": {
                                "ID": "wep-empty",
                                "Sockets": null
                            }
                        }
                    }
                },
                {
                    "Subject": null,
                    "CharacterID": "agent-sova",
                    "Loadout": {
                        "Subject": "",
                        "Items": {
                            "wep-vandal": {
                                "ID": "wep-vandal",
                                "Sockets": {
                                    "bcef87d6-209b-46c6-8b19-fbe40bd95abc": {
                                        "Item": { "ItemID": "skin-sakura" }
                                    }
                                }
                            }
                        }
                    }
                }
            ]
        }"#;
        let parsed: RawCoreGameLoadoutsResp = serde_json::from_str(raw_cg_json).expect("must handle nulls & top-level Subject");
        assert_eq!(parsed.loadouts.len(), 2);

        let p1 = resolve_coregame_loadout_for_player(&parsed.loadouts, "p-top-level", "agent-jett", 0, 2);
        let p1_skins = extract_skins_for_subject(&cache, p1);
        assert_eq!(p1_skins.len(), 1);
        assert_eq!(p1_skins[0].skin_name, "Kuronami Vandal");
        assert_eq!(p1_skins[0].skin_icon.as_deref(), Some("https://cdn.example/kuro-purple.png"));

        // Player 2 has blank Subject -> matched cleanly by unique CharacterID AND falls back to Deluxe skin
        let p2 = resolve_coregame_loadout_for_player(&parsed.loadouts, "p-sova-unknown", "agent-sova", 1, 2);
        let p2_skins = extract_skins_for_subject(&cache, p2);
        assert_eq!(p2_skins.len(), 1);
        assert_eq!(p2_skins[0].skin_name, "Sakura Vandal");
        assert_eq!(p2_skins[0].tier_name.as_deref(), Some("Deluxe"));

        // Pre-Game loadouts: handles null Items gracefully when LoadoutsValid=false AND parses skins when LoadoutsValid=true
        let raw_pg_invalid = r#"{"Loadouts":[{"Sprays":{"SpraySelections":null},"Items":null}],"LoadoutsValid":false}"#;
        let pg_inv: RawPreGameLoadoutsResp = serde_json::from_str(raw_pg_invalid).unwrap();
        assert!(!pg_inv.loadouts_valid);
        assert!(pg_inv.loadouts[0].items.is_empty());

        let raw_pg_valid = r#"{
            "LoadoutsValid": true,
            "Loadouts": [{
                "Subject": "p-ally-1",
                "Items": {
                    "wep-vandal": {
                        "ID": "wep-vandal",
                        "Sockets": {
                            "bcef87d6-209b-46c6-8b19-fbe40bd95abc": {
                                "Item": { "ID": "skin-kuro" }
                            }
                        }
                    }
                }
            }]
        }"#;
        let pg_val: RawPreGameLoadoutsResp = serde_json::from_str(raw_pg_valid).unwrap();
        assert!(pg_val.loadouts_valid);
        let ally_skins = extract_skins_from_items(&cache, &pg_val.loadouts[0].items);
        assert_eq!(ally_skins.len(), 1);
        assert_eq!(ally_skins[0].skin_name, "Kuronami Vandal");
    }

    #[test]
    fn test_reperspective_match_detail_and_known_player_search() {
        use crate::cache::reperspective_match_detail;
        use crate::models::career::{MatchDetailSnapshot, ScoreboardPlayer};

        let detail = MatchDetailSnapshot {
            match_id: "m-100".into(),
            my_team_id: "Blue".into(),
            my_team_score: 13,
            enemy_team_score: 7,
            won: true,
            scoreboard: vec![
                ScoreboardPlayer {
                    puuid: "p-blue".into(),
                    display_name: "BlueGuy#AP".into(),
                    team_id: "Blue".into(),
                    is_me: true,
                    ..Default::default()
                },
                ScoreboardPlayer {
                    puuid: "p-red".into(),
                    display_name: "RedGuy#AP".into(),
                    team_id: "Red".into(),
                    is_me: false,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let flipped = reperspective_match_detail(detail, "P-RED");
        assert_eq!(flipped.my_team_id, "Red");
        assert_eq!(flipped.my_team_score, 7);
        assert_eq!(flipped.enemy_team_score, 13);
        assert!(!flipped.won);
        assert!(!flipped.scoreboard[0].is_me);
        assert!(flipped.scoreboard[1].is_me);

        let cache = LocalCache::open(":memory:").unwrap();
        cache
            .put_cached_names(&[
                ("p-1".into(), "AlphaTest".into(), "AAA1".into()),
                ("p-2".into(), "BetaTest".into(), "BBB2".into()),
            ])
            .unwrap();
        let found = cache.search_known_players("betatest", 10).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].display_name, "BetaTest#BBB2");

        // Verify Riot ID parsing + lookup API key storage + friendly RESOURCE_NOT_FOUND formatting
        use crate::career::parse_riot_id;
        use crate::rate_limit::format_riot_api_error;
        assert_eq!(parse_riot_id("TenZ # SEN").unwrap(), ("TenZ".into(), "SEN".into()));
        assert_eq!(parse_riot_id("Player One#AP").unwrap(), ("Player One".into(), "AP".into()));
        assert!(parse_riot_id("NoTagName").is_err());
        assert!(parse_riot_id("OnlyHash#").is_err());

        assert!(cache.get_lookup_api_key().is_none());
        cache.set_lookup_api_key("HDEV-test-key").unwrap();
        assert_eq!(cache.get_lookup_api_key().as_deref(), Some("HDEV-test-key"));
        cache.set_lookup_api_key("").unwrap();
        assert!(cache.get_lookup_api_key().is_none());

        let friendly = format_riot_api_error(
            404,
            "/parties/v1/players/joinbycode/ABC",
            r#"{"httpStatus":404,"errorCode":"RESOURCE_NOT_FOUND","message":"status=404, detail=resource not found"}"#,
        );
        assert!(friendly.contains("open Valorant on PC"), "got: {friendly}");

        // Verify minimap view angle projection (Ascent: xMult > 0, yMult < 0 -> r=0 points UP (-pi/2), r=pi/2 points RIGHT (0))
        use crate::career::project_view_angle_rad;
        let ascent = MapInfo {
            x_multiplier: 0.00007,
            y_multiplier: -0.00007,
            x_scalar_to_add: 0.813895,
            y_scalar_to_add: 0.573242,
            ..Default::default()
        };
        let up = project_view_angle_rad(0.0, &ascent);
        assert!((up - (-std::f64::consts::FRAC_PI_2)).abs() < 1e-6, "up={up}");
        let right = project_view_angle_rad(std::f64::consts::FRAC_PI_2, &ascent);
        assert!(right.abs() < 1e-6, "right={right}");

        // Verify 15s fair-play remote agent hover & lock guard (blocks early hover/lock, allows >= 15s)
        use crate::live::{
            validate_pregame_action_delay, validate_pregame_lock_delay, FAIR_PLAY_LOCK_DELAY_SECS,
        };
        assert_eq!(FAIR_PLAY_LOCK_DELAY_SECS, 15);
        let early_hover = validate_pregame_action_delay(0, false).unwrap_err().to_string();
        assert!(early_hover.contains("Wait 15s before hovering"), "got: {early_hover}");
        let almost_hover = validate_pregame_action_delay(14, false).unwrap_err().to_string();
        assert!(almost_hover.contains("Wait 1s before hovering"), "got: {almost_hover}");
        assert!(validate_pregame_action_delay(15, false).is_ok());

        let early = validate_pregame_lock_delay(0).unwrap_err().to_string();
        assert!(early.contains("Wait 15s before locking"), "got: {early}");
        let almost = validate_pregame_lock_delay(14).unwrap_err().to_string();
        assert!(almost.contains("Wait 1s before locking"), "got: {almost}");
        assert!(validate_pregame_lock_delay(15).is_ok());
        assert!(validate_pregame_lock_delay(45).is_ok());
    }

    #[test]
    fn test_recent_comp_outcomes_and_mmr_cache_auto_heal() {
        use crate::models::mmr::PlayerMmrSummary;
        use crate::pd::{extract_recent_comp_outcomes, RawCompUpdatesLightResponse};

        let raw_json = r#"{
            "Matches": [
                { "MatchID": "m1", "TierAfterUpdate": 15, "TierBeforeUpdate": 15, "RankedRatingEarned": 22 },
                { "MatchID": "m2", "TierAfterUpdate": 15, "TierBeforeUpdate": 15, "RankedRatingEarned": -17 },
                { "MatchID": "m3", "TierAfterUpdate": 15, "TierBeforeUpdate": 15, "RankedRatingEarned": 0 },
                { "MatchID": "m-place", "TierAfterUpdate": 0, "TierBeforeUpdate": 0, "RankedRatingEarned": 0 },
                { "MatchID": "m4", "TierAfterUpdate": 16, "TierBeforeUpdate": 15, "RankedRatingEarned": 0 },
                { "MatchID": "m5", "TierAfterUpdate": 14, "TierBeforeUpdate": 15, "RankedRatingEarned": 0 },
                { "MatchID": "m6", "TierAfterUpdate": 15, "TierBeforeUpdate": 15, "RankedRatingEarned": 19 }
            ]
        }"#;
        let parsed: RawCompUpdatesLightResponse = serde_json::from_str(raw_json).unwrap();
        let outcomes = extract_recent_comp_outcomes(&parsed.matches, 5);
        // Win (+22), Loss (-17), Draw (0 same tier), skips unranked placement, Promotion (0), Demotion (0); capped at 5
        assert_eq!(outcomes, vec![1, -1, 0, 1, -1]);

        let cache = LocalCache::open(":memory:").unwrap();
        // Legacy cached row without recent_results_loaded must trigger cache miss so it auto-heals
        let legacy = PlayerMmrSummary {
            puuid: "P-LEGACY-UPPER".into(),
            recent_results: vec![],
            recent_outcomes: vec![],
            recent_results_loaded: false,
            ..Default::default()
        };
        cache.put_cached_mmr(&legacy).unwrap();
        assert!(cache.get_cached_mmr("p-legacy-upper", 600).is_none());

        // Fresh row with recent_results_loaded=true resolves case-insensitively
        let fresh = PlayerMmrSummary {
            puuid: "P-FRESH-UPPER".into(),
            recent_results: vec![true, false, false],
            recent_outcomes: vec![1, -1, 0],
            recent_results_loaded: true,
            ..Default::default()
        };
        cache.put_cached_mmr(&fresh).unwrap();
        let loaded = cache
            .get_cached_mmr("p-fresh-upper", 600)
            .expect("fresh row must hit cache");
        assert_eq!(loaded.recent_outcomes, vec![1, -1, 0]);
        assert!(loaded.recent_results_loaded);
    }

    #[test]
    fn test_enrich_loadout_picks_equipped_chroma_and_search_skins_sorts_popular_first() {
        use std::collections::HashSet;
        use crate::loadout::{
            apply_skin_to_raw_loadout, enrich_loadout, fetch_or_build_offline_loadout, RawLoadoutGun,
        };
        use crate::models::assets::{SkinChroma, SkinLevel, Weapon};

        let cache = LocalCache::open(":memory:").unwrap();

        // When no real loadout has been synced/cached yet, offline fallback MUST fail cleanly (no fake loadout)
        assert!(fetch_or_build_offline_loadout(&cache, "p-uncached").is_err());

        let vandal = Weapon {
            uuid: "wep-vandal".into(),
            display_name: "Vandal".into(),
            category: "Rifle".into(),
            ..Default::default()
        };
        cache.replace_category("weapon", &[vandal], |w| w.uuid.clone()).unwrap();

        let odin_skin = Skin {
            uuid: "skin-odin".into(),
            display_name: "Altitude Odin".into(),
            weapon_uuid: "wep-vandal-other".into(),
            weapon_name: "Odin".into(),
            content_tier_name: Some("Deluxe Edition".into()),
            display_icon: Some("https://cdn.example/odin.png".into()),
            ..Default::default()
        };
        let vandal_skin = Skin {
            uuid: "skin-kuro".into(),
            display_name: "Kuronami Vandal".into(),
            weapon_uuid: "wep-vandal".into(),
            weapon_name: "Vandal".into(),
            content_tier_name: Some("Exclusive Edition".into()),
            content_tier_color: "#F5955B".into(),
            display_icon: Some("https://cdn.example/kuro-base.png".into()),
            chromas: vec![
                SkinChroma {
                    uuid: "chroma-base".into(),
                    display_name: "Default".into(),
                    display_icon: Some("https://cdn.example/kuro-base.png".into()),
                    swatch: None,
                    streamed_video: None,
                },
                SkinChroma {
                    uuid: "CHROMA-WHITE".into(),
                    display_name: "White".into(),
                    display_icon: Some("https://cdn.example/kuro-white.png".into()),
                    swatch: None,
                    streamed_video: None,
                },
            ],
            levels: vec![SkinLevel {
                uuid: "lvl-4".into(),
                display_name: "L4".into(),
                display_icon: None,
                streamed_video: None,
            }],
            ..Default::default()
        };
        cache
            .replace_category("skin", &[odin_skin.clone(), vandal_skin.clone()], |s| s.uuid.clone())
            .unwrap();
        cache.rebuild_skin_level_index(&[odin_skin, vandal_skin]).unwrap();

        // Empty-query Explore catalog must rank Vandal Exclusive ahead of Odin Deluxe
        let catalog = cache.search_skins("", false, 50).unwrap();
        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog[0].display_name, "Kuronami Vandal");
        assert_eq!(catalog[1].display_name, "Altitude Odin");

        // Enrich loadout must resolve equipped chroma_id (case-insensitive) over base skin icon
        let mut raw = RawPlayerLoadout {
            subject: "p-1".into(),
            guns: vec![RawLoadoutGun {
                id: "wep-vandal".into(),
                skin_id: "skin-kuro".into(),
                skin_level_id: "lvl-4".into(),
                chroma_id: "chroma-white".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let snap = enrich_loadout(&cache, &raw);
        assert_eq!(
            snap.guns[0].skin_icon.as_deref(),
            Some("https://cdn.example/kuro-white.png")
        );

        // Re-applying the same skin must preserve the user's equipped chroma instead of resetting to base
        let owned = HashSet::from(["skin-kuro".to_string()]);
        let owned_lvl = HashSet::from(["lvl-4".to_string()]);
        apply_skin_to_raw_loadout(&cache, &mut raw, "wep-vandal", "skin-kuro", &owned, &owned_lvl, None, &HashSet::new(), None).unwrap();
        assert_eq!(raw.guns[0].chroma_id, "chroma-white");
    }

    #[test]
    fn test_loadout_extra_expressions_chroma_ownership_and_random_favorite_live_sync() {
        use std::collections::{HashMap, HashSet};
        use crate::cache::{CachedLiveMatchGuns, LocalCache};
        use crate::glz::{
            RawCoreGameLoadoutEntry, RawCoreLoadoutDetail, RawCoreLoadoutItem,
            RawCoreLoadoutSocket, RawSocketItem,
        };
        use crate::live::{extract_live_gun_map, friendly_server_pod};
        use crate::loadout::{
            apply_skin_to_raw_loadout, enrich_loadout, RawLoadoutGun, RawLoadoutPutBody,
            RawPlayerLoadout,
        };
        use crate::models::assets::{Skin, SkinChroma, SkinLevel, Spray, Weapon};

        // 1. Unknown top-level Riot fields like Expressions (Flex) must survive GET -> PUT round-trip
        //    while Subject and Version are stripped on the PUT body.
        let raw_json = r#"{
            "Subject": "puuid-expr",
            "Version": 42,
            "Guns": [],
            "Sprays": [],
            "Expressions": { "AESSelections": [{ "SocketID": "flex-1", "AssetID": "expr-asset-1" }] },
            "Incognito": false
        }"#;
        let parsed: RawPlayerLoadout = serde_json::from_str(raw_json).unwrap();
        assert!(parsed.extra.contains_key("Expressions"));

        let mut extra = parsed.extra.clone();
        extra.retain(|k, _| !k.eq_ignore_ascii_case("Subject") && !k.eq_ignore_ascii_case("Version"));
        let put_body = RawLoadoutPutBody {
            guns: parsed.guns.clone(),
            sprays: parsed.sprays.clone(),
            identity: parsed.identity.clone(),
            incognito: parsed.incognito,
            extra,
        };
        let put_val = serde_json::to_value(&put_body).unwrap();
        assert!(put_val.get("Expressions").is_some());
        assert!(put_val.get("Subject").is_none());
        assert!(put_val.get("Version").is_none());

        // 2. Chroma ownership checks + Random Favorite live-match resolution + canonical Spray order
        let cache = LocalCache::open(":memory:").unwrap();
        let vandal = Weapon {
            uuid: "wep-vandal".into(),
            display_name: "Vandal".into(),
            category: "Rifle".into(),
            display_icon: Some("https://cdn.example/vandal-default.png".into()),
            ..Default::default()
        };
        cache.replace_category("weapon", &[vandal], |w| w.uuid.clone()).unwrap();

        let random_skin = Skin {
            uuid: "skin-random".into(),
            display_name: "Random Favorite Weapon Skin".into(),
            weapon_uuid: "wep-vandal".into(),
            weapon_name: "Vandal".into(),
            content_tier_color: "#5A9FE2".into(),
            chromas: vec![],
            levels: vec![],
            ..Default::default()
        };
        let reaver_skin = Skin {
            uuid: "skin-reaver".into(),
            display_name: "Reaver Vandal".into(),
            weapon_uuid: "wep-vandal".into(),
            weapon_name: "Vandal".into(),
            content_tier_color: "#D1548D".into(),
            display_icon: Some("https://cdn.example/reaver-base.png".into()),
            chromas: vec![
                SkinChroma {
                    uuid: "chroma-base".into(),
                    display_name: "Default".into(),
                    display_icon: Some("https://cdn.example/reaver-base.png".into()),
                    swatch: None,
                    streamed_video: None,
                },
                SkinChroma {
                    uuid: "chroma-red".into(),
                    display_name: "Red".into(),
                    display_icon: Some("https://cdn.example/reaver-red.png".into()),
                    swatch: None,
                    streamed_video: None,
                },
                SkinChroma {
                    uuid: "chroma-black".into(),
                    display_name: "Black".into(),
                    display_icon: Some("https://cdn.example/reaver-black.png".into()),
                    swatch: None,
                    streamed_video: None,
                },
            ],
            levels: vec![SkinLevel {
                uuid: "lvl-reaver-4".into(),
                display_name: "Level 4".into(),
                display_icon: None,
                streamed_video: None,
            }],
            ..Default::default()
        };
        cache
            .replace_category("skin", &[random_skin.clone(), reaver_skin.clone()], |s| s.uuid.clone())
            .unwrap();
        cache.rebuild_skin_level_index(&[random_skin, reaver_skin]).unwrap();

        let sprays = vec![
            Spray { uuid: "sp-1".into(), display_name: "GLHF".into(), display_icon: Some("https://cdn.example/sp1.png".into()), full_transparent_icon: None },
            Spray { uuid: "sp-2".into(), display_name: "GGWP".into(), display_icon: Some("https://cdn.example/sp2.png".into()), full_transparent_icon: Some("https://cdn.example/sp2t.png".into()) },
        ];
        cache.replace_category("spray", &sprays, |s| s.uuid.clone()).unwrap();

        let mut loadout = RawPlayerLoadout {
            subject: "p-live".into(),
            guns: vec![RawLoadoutGun {
                id: "wep-vandal".into(),
                skin_id: "skin-random".into(),
                ..Default::default()
            }],
            sprays: vec![
                crate::loadout::RawLoadoutSpray {
                    equip_slot_id: "5863985e-43ac-b05d-cb2d-139e72970014".into(), // Post-Round
                    spray_id: "sp-2".into(),
                    ..Default::default()
                },
                crate::loadout::RawLoadoutSpray {
                    equip_slot_id: "0814b2fe-4512-60a4-5288-1fbdcec6ca48".into(), // Pre-Round
                    spray_id: "sp-1".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // Extract live match guns and cache them -> enriches Random Favorite into Reaver Red!
        let mut cg_sockets = HashMap::new();
        cg_sockets.insert(
            "bcef87d6-209b-46c6-8b19-fbe40bd95abc".into(),
            RawCoreLoadoutSocket {
                id: "bcef87d6-209b-46c6-8b19-fbe40bd95abc".into(),
                item: RawSocketItem { id: "skin-reaver".into(), type_id: String::new() },
            },
        );
        cg_sockets.insert(
            "3ad1b2b2-acdb-4524-852f-954a77ddae05".into(),
            RawCoreLoadoutSocket {
                id: "3ad1b2b2-acdb-4524-852f-954a77ddae05".into(),
                item: RawSocketItem { id: "chroma-red".into(), type_id: String::new() },
            },
        );
        let cg_entry = RawCoreGameLoadoutEntry {
            subject: "p-live".into(),
            character_id: String::new(),
            loadout: RawCoreLoadoutDetail {
                subject: "p-live".into(),
                items: HashMap::from([(
                    "wep-vandal".into(),
                    RawCoreLoadoutItem {
                        id: "wep-vandal".into(),
                        type_id: String::new(),
                        sockets: cg_sockets,
                    },
                )]),
                ..Default::default()
            },
            items: HashMap::new(),
        };
        let live_guns = extract_live_gun_map(Some(&cg_entry));
        cache
            .put_live_match_guns(
                "p-live",
                &CachedLiveMatchGuns { match_id: "m-1".into(), guns: live_guns, sprays: vec![] },
            )
            .unwrap();

        let snap_live = enrich_loadout(&cache, &loadout);
        assert!(snap_live.is_live_match_synced);
        assert!(snap_live.guns[0].is_live_match_skin);
        assert_eq!(snap_live.guns[0].skin_name, "Reaver Vandal");
        assert_eq!(snap_live.guns[0].chroma_uuid, "chroma-red");
        assert_eq!(snap_live.guns[0].skin_icon.as_deref(), Some("https://cdn.example/reaver-red.png"));
        // Canonical spray ordering puts Pre-Round first, Mid-Round second, Post-Round third, Extra fourth
        assert_eq!(snap_live.sprays.len(), 4);
        assert_eq!(snap_live.sprays[0].slot_label, "Pre-Round");
        assert_eq!(snap_live.sprays[1].slot_label, "Mid-Round");
        assert_eq!(snap_live.sprays[1].spray_name, "Empty");
        assert_eq!(snap_live.sprays[2].slot_label, "Post-Round");
        assert_eq!(snap_live.sprays[2].spray_icon.as_deref(), Some("https://cdn.example/sp2t.png"));
        assert_eq!(snap_live.sprays[3].slot_label, "Extra");

        cache.clear_live_match_guns("p-live");
        assert!(cache.get_live_match_guns("p-live").is_none());

        // Owned vs unowned chroma enforcement
        let owned_skins = HashSet::from(["skin-reaver".to_string()]);
        let owned_lvls = HashSet::from(["lvl-reaver-4".to_string()]);
        let owned_chromas = HashSet::from(["chroma-red".to_string()]);

        // Base chroma (idx 0) and owned chroma ("chroma-red") succeed
        apply_skin_to_raw_loadout(&cache, &mut loadout, "wep-vandal", "skin-reaver", &owned_skins, &owned_lvls, Some("chroma-base"), &owned_chromas, None).unwrap();
        assert_eq!(loadout.guns[0].chroma_id, "chroma-base");
        apply_skin_to_raw_loadout(&cache, &mut loadout, "wep-vandal", "skin-reaver", &owned_skins, &owned_lvls, Some("chroma-red"), &owned_chromas, None).unwrap();
        assert_eq!(loadout.guns[0].chroma_id, "chroma-red");

        // Unowned non-base chroma ("chroma-black") must fail cleanly
        assert!(apply_skin_to_raw_loadout(&cache, &mut loadout, "wep-vandal", "skin-reaver", &owned_skins, &owned_lvls, Some("chroma-black"), &owned_chromas, None).is_err());

        // 3. Server pod friendly labels
        assert_eq!(friendly_server_pod("aresriot.aws-bom1-prod.ap-gp-mumbai-1"), "Mumbai");
        assert_eq!(friendly_server_pod("aresriot.aws-ap-southeast-1-prod.ap-gp-singapore-1"), "Singapore");
        assert_eq!(friendly_server_pod("aresriot.aws-eu-central-1-prod.eu-gp-frankfurt-1"), "Frankfurt");
    }

    #[test]
    fn test_live_override_non_random_expressions_flex_level_and_cosmetic_mutators() {
        use std::collections::{HashMap, HashSet};
        use crate::cache::{CachedLiveMatchGuns, LocalCache};
        use crate::loadout::{
            apply_identity_privacy_to_raw_loadout, apply_player_card_to_raw_loadout,
            apply_player_title_to_raw_loadout, apply_skin_to_raw_loadout,
            apply_spray_to_raw_loadout, enrich_loadout, RawPlayerLoadout,
        };
        use crate::models::assets::{FlexAsset, Skin, SkinChroma, SkinLevel, Spray, Weapon};

        let cache = LocalCache::open(":memory:").unwrap();
        let vandal = Weapon {
            uuid: "wep-vandal".into(),
            display_name: "Vandal".into(),
            category: "Rifle".into(),
            ..Default::default()
        };
        cache.replace_category("weapon", &[vandal], |w| w.uuid.clone()).unwrap();

        let next_skin = Skin {
            uuid: "skin-champs".into(),
            display_name: "Champions 2025 Vandal".into(),
            weapon_uuid: "wep-vandal".into(),
            weapon_name: "Vandal".into(),
            content_tier_color: "#F0B941".into(),
            levels: vec![
                SkinLevel { uuid: "lvl-c1".into(), display_name: "Level 1".into(), display_icon: Some("https://cdn.example/c1.png".into()), streamed_video: None },
                SkinLevel { uuid: "lvl-c2".into(), display_name: "Level 2".into(), display_icon: None, streamed_video: None },
            ],
            ..Default::default()
        };
        let live_skin = Skin {
            uuid: "skin-kuronami".into(),
            display_name: "Kuronami Vandal".into(),
            weapon_uuid: "wep-vandal".into(),
            weapon_name: "Vandal".into(),
            content_tier_color: "#D1548D".into(),
            chromas: vec![SkinChroma {
                uuid: "chr-kuro-base".into(),
                display_name: "Base".into(),
                display_icon: Some("https://cdn.example/kuro.png".into()),
                swatch: None,
                streamed_video: None,
            }],
            levels: vec![SkinLevel {
                uuid: "lvl-kuro-1".into(),
                display_name: "Level 1".into(),
                display_icon: Some("https://cdn.example/kuro-lvl1.png".into()),
                streamed_video: None,
            }],
            ..Default::default()
        };
        cache
            .replace_category("skin", &[next_skin.clone(), live_skin.clone()], |s| s.uuid.clone())
            .unwrap();
        cache.rebuild_skin_level_index(&[next_skin, live_skin]).unwrap();

        let spray = Spray {
            uuid: "sp-aes".into(),
            display_name: "VCT Hype".into(),
            display_icon: Some("https://cdn.example/sp_aes.png".into()),
            full_transparent_icon: None,
        };
        cache.replace_category("spray", &[spray], |s| s.uuid.clone()).unwrap();
        let flex = FlexAsset {
            uuid: "flex-1".into(),
            display_name: "tactibear Flex".into(),
            display_icon: Some("https://cdn.example/flex1.png".into()),
        };
        cache.replace_category("flex", &[flex], |f| f.uuid.clone()).unwrap();

        // Raw loadout has a CONCRETE (non-Random-Favorite) next-match skin + Riot Expressions.AESSelections
        let raw_json = r#"{
            "Subject": "p-concrete",
            "Version": 7,
            "Guns": [{
                "ID": "wep-vandal",
                "SkinID": "skin-champs",
                "SkinLevelID": "lvl-c1",
                "ChromaID": ""
            }],
            "Sprays": [],
            "Expressions": {
                "AESSelections": [
                    { "SocketID": "0814b2fe-4512-60a4-5288-1fbdcec6ca48", "AssetID": "sp-aes" },
                    { "SocketID": "c9f11111-1111-1111-1111-111111111111", "AssetID": "flex-1" }
                ]
            },
            "Identity": { "PlayerCardID": "card-1", "PlayerTitleID": "title-1", "HideAccountLevel": false },
            "Incognito": false
        }"#;
        let mut raw: RawPlayerLoadout = serde_json::from_str(raw_json).unwrap();

        // Live match says Kuronami Vandal was actually locked into the active game
        let mut live_guns = HashMap::new();
        live_guns.insert(
            "wep-vandal".to_string(),
            ("skin-kuronami".to_string(), "lvl-kuro-1".to_string(), "chr-kuro-base".to_string()),
        );
        cache
            .put_live_match_guns(
                "p-concrete",
                &CachedLiveMatchGuns { match_id: "m-live".into(), guns: live_guns, sprays: vec![] },
            )
            .unwrap();

        let snap = enrich_loadout(&cache, &raw);
        assert!(snap.is_live_match_synced);
        assert!(snap.guns[0].is_live_match_skin);
        assert_eq!(snap.guns[0].skin_name, "Kuronami Vandal");
        assert_eq!(snap.guns[0].chroma_uuid, "chr-kuro-base");
        assert_eq!(snap.guns[0].level_uuid, "lvl-kuro-1");
        assert_eq!(snap.guns[0].next_match_skin_name.as_deref(), Some("Champions 2025 Vandal"));

        // Spray + Flex resolved from Expressions.AESSelections alongside padded canonical slots
        assert_eq!(snap.sprays.len(), 4);
        assert_eq!(snap.sprays[0].spray_name, "VCT Hype");
        assert!(!snap.sprays[0].is_flex);
        let flex_slot = snap.sprays.iter().find(|s| s.is_flex).expect("flex slot preserved");
        assert_eq!(flex_slot.spray_name, "tactibear Flex");

        // Owned vs unowned explicit level selection
        let owned_skins = HashSet::from(["skin-champs".to_string()]);
        let owned_lvls = HashSet::from(["lvl-c1".to_string()]);
        apply_skin_to_raw_loadout(&cache, &mut raw, "wep-vandal", "skin-champs", &owned_skins, &owned_lvls, None, &HashSet::new(), Some("lvl-c1")).unwrap();
        assert_eq!(raw.guns[0].skin_level_id, "lvl-c1");
        assert!(apply_skin_to_raw_loadout(&cache, &mut raw, "wep-vandal", "skin-champs", &owned_skins, &owned_lvls, None, &HashSet::new(), Some("lvl-c2")).is_err());

        // Spray, Card, Title, Privacy mutators
        let owned_sprays = HashSet::from(["sp-aes".to_string(), "sp-new".to_string()]);
        apply_spray_to_raw_loadout(&mut raw, "0814b2fe-4512-60a4-5288-1fbdcec6ca48", "sp-new", &owned_sprays).unwrap();
        assert_eq!(raw.sprays.len(), 1);
        assert_eq!(raw.sprays[0].spray_id, "sp-new");
        // Verify AESSelections also synced
        let aes = raw.extra["Expressions"]["AESSelections"].as_array().unwrap();
        assert_eq!(aes[0]["AssetID"].as_str(), Some("sp-new"));
        assert!(apply_spray_to_raw_loadout(&mut raw, "0814b2fe-4512-60a4-5288-1fbdcec6ca48", "sp-unowned", &owned_sprays).is_err());

        let owned_cards = HashSet::from(["card-2".to_string()]);
        apply_player_card_to_raw_loadout(&mut raw, "card-2", &owned_cards).unwrap();
        assert_eq!(raw.identity.player_card_id, "card-2");
        assert!(apply_player_card_to_raw_loadout(&mut raw, "card-unowned", &owned_cards).is_err());

        let owned_titles = HashSet::from(["title-2".to_string()]);
        apply_player_title_to_raw_loadout(&mut raw, "title-2", &owned_titles).unwrap();
        assert_eq!(raw.identity.player_title_id, "title-2");
        apply_player_title_to_raw_loadout(&mut raw, "", &owned_titles).unwrap();
        assert_eq!(raw.identity.player_title_id, "");
        assert!(apply_player_title_to_raw_loadout(&mut raw, "title-unowned", &owned_titles).is_err());

        apply_identity_privacy_to_raw_loadout(&mut raw, true, true);
        assert!(raw.incognito);
        assert!(raw.identity.hide_account_level);
    }

    #[test]
    fn test_personalization_v3_active_expressions_entitlements_shapes_and_404_scoping() {
        use std::collections::{HashMap, HashSet};
        use crate::cache::LocalCache;
        use crate::loadout::{
            apply_spray_to_raw_loadout, enrich_loadout, RawEntitlementsResp, RawLoadoutPutBodyV3,
            RawPlayerLoadout,
        };
        use crate::models::assets::{FlexAsset, Spray};
        use crate::pd::{ITEM_TYPE_FLEX, ITEM_TYPE_SPRAY};
        use crate::rate_limit::format_riot_api_error;

        let cache = LocalCache::open(":memory:").unwrap();
        let sprays = vec![
            Spray { uuid: "sp-1".into(), display_name: "GLHF".into(), display_icon: Some("https://cdn.example/s1.png".into()), full_transparent_icon: None },
            Spray { uuid: "sp-2".into(), display_name: "GG".into(), display_icon: Some("https://cdn.example/s2.png".into()), full_transparent_icon: None },
            Spray { uuid: "sp-3".into(), display_name: "Nice Try".into(), display_icon: Some("https://cdn.example/s3.png".into()), full_transparent_icon: None },
        ];
        cache.replace_category("spray", &sprays, |s| s.uuid.clone()).unwrap();
        let flex = FlexAsset {
            uuid: "flex-panda".into(),
            display_name: "Bamboo Buddy".into(),
            display_icon: Some("https://cdn.example/flex.png".into()),
        };
        cache.replace_category("flex", &[flex], |f| f.uuid.clone()).unwrap();

        let v3_json = format!(
            r#"{{
                "Subject": "puuid-v3",
                "Version": 19,
                "Guns": [],
                "ActiveExpressions": [
                    {{ "TypeID": "{ITEM_TYPE_SPRAY}", "AssetID": "sp-1" }},
                    {{ "TypeID": "{ITEM_TYPE_SPRAY}", "AssetID": "sp-2" }},
                    {{ "TypeID": "{ITEM_TYPE_FLEX}", "AssetID": "flex-panda" }}
                ],
                "DynamicOptions": {{ "PreferredMode": "comp" }},
                "Identity": {{ "PlayerCardID": "card-1", "PlayerTitleID": "", "AccountLevel": 500, "PreferredLevelBorderID": "", "HideAccountLevel": false }},
                "Incognito": false
            }}"#
        );
        let mut raw: RawPlayerLoadout = serde_json::from_str(&v3_json).unwrap();
        raw.api_version = 3;

        let snap = enrich_loadout(&cache, &raw);
        assert_eq!(snap.sprays.len(), 5);
        assert_eq!(snap.sprays[0].slot_uuid, "expr:0");
        assert_eq!(snap.sprays[0].spray_name, "GLHF");
        assert_eq!(snap.sprays[1].spray_name, "GG");
        assert_eq!(snap.sprays[2].slot_uuid, "spray:2");
        assert_eq!(snap.sprays[2].spray_name, "Empty");
        assert_eq!(snap.sprays[3].slot_uuid, "spray:3");
        assert!(snap.sprays[4].is_flex);
        assert_eq!(snap.sprays[4].spray_name, "Bamboo Buddy");

        // Swapping spray slot 2 must preserve Flex at index 2 in ActiveExpressions
        let owned = HashSet::from(["sp-1".to_string(), "sp-2".to_string(), "sp-3".to_string()]);
        apply_spray_to_raw_loadout(&mut raw, "spray:2", "sp-3", &owned).unwrap();
        assert_eq!(raw.active_expressions.len(), 4);
        assert!(raw.active_expressions.iter().any(|e| e.type_id == ITEM_TYPE_FLEX && e.asset_id == "flex-panda"));
        apply_spray_to_raw_loadout(&mut raw, "expr:0", "sp-3", &owned).unwrap();
        assert_eq!(raw.active_expressions[0].asset_id, "sp-3");

        // Verify v3 PUT body serialization omits Sprays, Subject, Version, ApiVersion and includes ActiveExpressions & DynamicOptions
        let put_v3 = RawLoadoutPutBodyV3 {
            guns: raw.guns.clone(),
            active_expressions: raw.active_expressions.clone(),
            dynamic_options: raw.dynamic_options.clone().unwrap(),
            identity: raw.identity.clone(),
            incognito: raw.incognito,
            extra: HashMap::new(),
        };
        let put_val = serde_json::to_value(&put_v3).unwrap();
        assert!(put_val.get("ActiveExpressions").is_some());
        assert_eq!(put_val["DynamicOptions"]["PreferredMode"].as_str(), Some("comp"));
        assert!(put_val.get("Sprays").is_none());
        assert!(put_val.get("Subject").is_none());
        assert!(put_val.get("Version").is_none());

        // Entitlements parsing: flat Entitlements vs nested EntitlementsByTypes (with InstanceID)
        let flat_json = r#"{ "Entitlements": [{ "ItemID": "chroma-a" }] }"#;
        let flat: RawEntitlementsResp = serde_json::from_str(flat_json).unwrap();
        assert_eq!(flat.into_items(None).len(), 1);

        let grouped_json = r#"{
            "EntitlementsByTypes": [
                {
                    "ItemTypeID": "dd3bf334-87f3-40bd-b043-682a57a8dc3a",
                    "Entitlements": [{ "TypeID": "dd3bf334-87f3-40bd-b043-682a57a8dc3a", "ItemID": "buddy-1", "InstanceID": "inst-99" }]
                },
                {
                    "ItemTypeID": "3ad1b2b2-acdb-4524-852f-954a76ddae0a",
                    "Entitlements": [{ "ItemID": "chroma-red" }]
                }
            ]
        }"#;
        let grouped: RawEntitlementsResp = serde_json::from_str(grouped_json).unwrap();
        let chromas = grouped.into_items(Some("3ad1b2b2-acdb-4524-852f-954a76ddae0a"));
        assert_eq!(chromas.len(), 1);
        assert_eq!(chromas[0].item_id, "chroma-red");
        let grouped2: RawEntitlementsResp = serde_json::from_str(grouped_json).unwrap();
        let buddies = grouped2.into_items(Some("dd3bf334-87f3-40bd-b043-682a57a8dc3a"));
        assert_eq!(buddies[0].instance_id.as_deref(), Some("inst-99"));

        // 404 RESOURCE_NOT_FOUND formatting must be scoped to GLZ party/match routes
        let party_err = format_riot_api_error(
            404,
            "/parties/v1/players/abc",
            r#"{"errorCode":"RESOURCE_NOT_FOUND","message":"Player not found"}"#,
        );
        assert!(party_err.contains("open Valorant on PC first"));

        let loadout_err = format_riot_api_error(
            404,
            "/personalization/v2/players/abc/playerloadout",
            r#"{"errorCode":"RESOURCE_NOT_FOUND","message":"Route not found"}"#,
        );
        assert!(!loadout_err.contains("open Valorant on PC first"));
        assert!(loadout_err.contains("RESOURCE_NOT_FOUND"));
    }

    #[test]
    fn test_unicode_riot_id_lookup_in_name_cache() {
        let cache = LocalCache::open(":memory:").unwrap();
        cache
            .put_cached_names(&[(
                "puuid-unicode-1".to_string(),
                "VØIDΞN".to_string(),
                "EUW".to_string(),
            )])
            .unwrap();
        assert_eq!(
            cache.find_cached_puuid_by_riot_id("vøidξn", "euw").as_deref(),
            Some("puuid-unicode-1")
        );
        assert_eq!(
            cache.find_cached_puuid_by_riot_id("VØIDΞN", "EUW").as_deref(),
            Some("puuid-unicode-1")
        );
    }

    #[test]
    fn test_cached_storefront_elapsed_and_expired_eviction() {
        use crate::models::store::{FeaturedBundle, StorefrontSnapshot};

        let cache = LocalCache::open(":memory:").unwrap();
        let now = LocalCache::now_secs();
        let fresh = StorefrontSnapshot {
            fetched_at_epoch_secs: now - 600,
            daily_remaining_secs: 3600,
            accessory_remaining_secs: 7200,
            featured_bundles: vec![FeaturedBundle {
                remaining_secs: 1800,
                ..Default::default()
            }],
            ..Default::default()
        };
        cache.put_cached_storefront("Puuid-Case-1", &fresh).unwrap();
        let loaded = cache
            .get_cached_storefront("puuid-case-1")
            .unwrap()
            .expect("unexpired storefront should load case-insensitively");
        assert!((2995..=3005).contains(&loaded.daily_remaining_secs));
        assert!((6595..=6605).contains(&loaded.accessory_remaining_secs));
        assert!((1195..=1205).contains(&loaded.featured_bundles[0].remaining_secs));

        let expired = StorefrontSnapshot {
            fetched_at_epoch_secs: now - 4000,
            daily_remaining_secs: 3600,
            ..Default::default()
        };
        cache.put_cached_storefront("puuid-expired", &expired).unwrap();
        assert!(cache.get_cached_storefront("puuid-expired").unwrap().is_none());
    }

    #[test]
    fn test_save_loadout_preset_same_second_unique_ids() {
        let cache = LocalCache::open(":memory:").unwrap();
        let p1 = cache
            .save_loadout_preset("puuid-seq", "Alpha", "#FFFFFF", 5, &[], "{}")
            .unwrap();
        let p2 = cache
            .save_loadout_preset("puuid-seq", "Beta", "#00FFFF", 6, &[], "{}")
            .unwrap();
        assert_ne!(p1.preset_id, p2.preset_id);
        assert_eq!(cache.list_loadout_presets(Some("puuid-seq")).unwrap().len(), 2);
    }

    #[test]
    fn test_apply_spray_protects_flex_slot() {
        use crate::loadout::{
            apply_spray_to_raw_loadout, RawLoadoutExpression, RawLoadoutIdentity, RawPlayerLoadout,
        };
        use crate::pd::{ITEM_TYPE_FLEX, ITEM_TYPE_SPRAY};
        use std::collections::{HashMap, HashSet};

        let mut raw = RawPlayerLoadout {
            subject: "puuid-flex".into(),
            version: 1,
            api_version: 3,
            guns: vec![],
            sprays: vec![],
            active_expressions: vec![
                RawLoadoutExpression {
                    type_id: ITEM_TYPE_SPRAY.into(),
                    asset_id: "sp-1".into(),
                    extra: HashMap::new(),
                },
                RawLoadoutExpression {
                    type_id: ITEM_TYPE_SPRAY.into(),
                    asset_id: "sp-2".into(),
                    extra: HashMap::new(),
                },
                RawLoadoutExpression {
                    type_id: ITEM_TYPE_FLEX.into(),
                    asset_id: "flex-panda".into(),
                    extra: HashMap::new(),
                },
            ],
            dynamic_options: None,
            identity: RawLoadoutIdentity::default(),
            incognito: false,
            extra: HashMap::new(),
        };
        let owned = HashSet::from(["sp-1".to_string(), "sp-3".to_string()]);
        let res = apply_spray_to_raw_loadout(&mut raw, "expr:2", "sp-3", &owned);
        assert!(res.is_err());
        assert_eq!(raw.active_expressions[2].type_id, ITEM_TYPE_FLEX);
        assert_eq!(raw.active_expressions[2].asset_id, "flex-panda");
    }
}
