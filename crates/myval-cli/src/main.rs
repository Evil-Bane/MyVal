use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::Utc;
use myval_core::auth::{build_riot_headers, build_riot_login_url, region_to_shard};
use myval_core::models::auth::RiotSession;
use myval_core::models::live::LiveState;
use myval_core::pd::pd_base;
use myval_core::MyValEngineCore;

fn default_dev_dir() -> PathBuf {
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    base.join("MyValDev")
}

fn riot_lockfile_path() -> PathBuf {
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    base.join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile")
}

fn decode_jwt_payload(jwt: &str) -> Option<serde_json::Value> {
    let seg = jwt.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(seg)
        .or_else(|_| {
            let mut padded = seg.replace('-', "+").replace('_', "/");
            while padded.len() % 4 != 0 {
                padded.push('=');
            }
            STANDARD.decode(padded)
        })
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

async fn load_or_refresh_session_from_lockfile(
    session_path: &PathBuf,
) -> Result<(RiotSession, &'static str), String> {
    let lock_path = riot_lockfile_path();
    if lock_path.exists() {
        let raw_lock = fs::read_to_string(&lock_path)
            .map_err(|e| format!("Failed reading lockfile: {e}"))?;
        let parts: Vec<&str> = raw_lock.trim().split(':').collect();
        if parts.len() >= 4 {
            let port = parts[2];
            let password = parts[3];
            let basic = STANDARD.encode(format!("riot:{password}"));
            let local_http = reqwest::Client::builder()
                .http1_only()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(6))
                .build()
                .map_err(|e| format!("Loopback HTTP client error: {e}"))?;

            let ent_url = format!("https://127.0.0.1:{port}/entitlements/v1/token");
            let ent_resp_res = local_http
                .get(&ent_url)
                .header("Authorization", format!("Basic {basic}"))
                .send()
                .await;
            let ent_resp_opt = match ent_resp_res {
                Ok(resp) if resp.status().is_success() => Some(resp),
                Ok(resp) => {
                    eprintln!(
                        "Note: Local Riot Client lockfile returned HTTP {}; trying cached session if available.",
                        resp.status()
                    );
                    None
                }
                Err(e) => {
                    eprintln!(
                        "Note: Local Riot Client lockfile (port {port}) unavailable ({e}); trying cached session if available."
                    );
                    None
                }
            };
            if let Some(ent_resp) = ent_resp_opt {
                let ent_json: serde_json::Value = ent_resp
                    .json()
                    .await
                    .map_err(|e| format!("Invalid local entitlements JSON: {e}"))?;
                let access_token = ent_json
                    .get("accessToken")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "Local /entitlements/v1/token missing accessToken".to_string())?
                    .to_string();
                let entitlements_token = ent_json
                    .get("token")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "Local /entitlements/v1/token missing token".to_string())?
                    .to_string();
                let puuid = ent_json
                    .get("subject")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "Local /entitlements/v1/token missing subject".to_string())?
                    .to_string();

                let region_url = format!("https://127.0.0.1:{port}/riotclient/region-locale");
                let region_resp = local_http
                    .get(&region_url)
                    .header("Authorization", format!("Basic {basic}"))
                    .send()
                    .await
                    .ok();
                let region_val: Option<serde_json::Value> = match region_resp {
                    Some(r) if r.status().is_success() => r.json().await.ok(),
                    _ => None,
                };

                let alias_url = format!("https://127.0.0.1:{port}/player-account/aliases/v1/active");
                let alias_resp = local_http
                    .get(&alias_url)
                    .header("Authorization", format!("Basic {basic}"))
                    .send()
                    .await
                    .ok();
                let alias_val: Option<serde_json::Value> = match alias_resp {
                    Some(r) if r.status().is_success() => r.json().await.ok(),
                    _ => None,
                };

                let jwt_payload = decode_jwt_payload(&access_token);
                let jwt_region = jwt_payload
                    .as_ref()
                    .and_then(|p| p.get("pp"))
                    .and_then(|pp| pp.get("c"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let locale_region = region_val
                    .as_ref()
                    .and_then(|v| v.get("region").or_else(|| v.get("webRegion")))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let raw_region = jwt_region
                    .or(locale_region)
                    .unwrap_or_else(|| "ap".to_string());
                let mapped = region_to_shard(&raw_region);

                let expires_at = jwt_payload
                    .as_ref()
                    .and_then(|p| p.get("exp"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or_else(|| Utc::now().timestamp() + 3300);

                let game_name = alias_val
                    .as_ref()
                    .and_then(|v| v.get("game_name").or_else(|| v.get("gameName")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("LocalPlayer")
                    .to_string();
                let tag_line = alias_val
                    .as_ref()
                    .and_then(|v| v.get("tag_line").or_else(|| v.get("tagLine")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("LOCAL")
                    .to_string();

                let session = RiotSession {
                    puuid,
                    game_name,
                    tag_line,
                    region: mapped.region,
                    shard: mapped.shard,
                    access_token,
                    id_token: String::new(),
                    entitlements_token,
                    expires_at_epoch_secs: expires_at,
                };
                if let Some(parent) = session_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let json = serde_json::to_string(&session)
                    .map_err(|e| format!("Serialize session error: {e}"))?;
                fs::write(session_path, json)
                    .map_err(|e| format!("Failed saving cached session: {e}"))?;
                return Ok((session, "riot-lockfile"));
            }
        }
    }

    if session_path.exists() {
        let json = fs::read_to_string(session_path)
            .map_err(|e| format!("Failed reading cached session: {e}"))?;
        let session: RiotSession = serde_json::from_str(&json)
            .map_err(|e| format!("Invalid cached session JSON: {e}"))?;
        return Ok((session, "cached-session"));
    }

    Err(format!(
        "No active Riot Client lockfile at '{}' and no cached session at '{}'",
        lock_path.display(),
        session_path.display()
    ))
}

async fn run_probe(
    db_path: &PathBuf,
    session_path: &PathBuf,
    redirect_url: Option<&str>,
    test_put: bool,
    explore_extra: bool,
) {
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let engine = MyValEngineCore::open(db_path.to_str().unwrap_or("dev.db"))
        .expect("failed to open dev sqlite db");

    let (session, source) = if let Some(url) = redirect_url {
        let s = match engine.complete_auth_from_redirect_url(url).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("PROBE RSO ERROR: {e}");
                process::exit(1);
            }
        };
        if let Some(parent) = session_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(Some(json)) = engine.export_session_json().await {
            if let Err(e) = fs::write(session_path, json) {
                eprintln!("Warning: Failed saving cached session: {e}");
            }
        }
        (s, "redirect-url")
    } else {
        match load_or_refresh_session_from_lockfile(session_path).await {
            Ok((s, src)) => {
                let session_json = serde_json::to_string(&s).unwrap();
                engine
                    .restore_session_json(&session_json)
                    .await
                    .expect("restore session failed");
                (s, src)
            }
            Err(e) => {
                eprintln!("PROBE AUTH ERROR: {e}");
                process::exit(1);
            }
        }
    };

    println!(
        "=== MyVal Dev Probe ===\nSession source : {source}\nPlayer         : {}#{} (region={}, shard={}, puuid={}...)\nSession file   : {} [redacted from stdout]",
        session.game_name,
        session.tag_line,
        session.region,
        session.shard,
        session.puuid.chars().take(8).collect::<String>(),
        session_path.display()
    );

    if engine.needs_asset_resync() {
        println!("Syncing valorant-api.com catalog...");
        let summary = engine.sync_assets().await.expect("asset sync failed");
        println!(
            "Assets synced  : clientVersion={}, skins={}, weapons={}, agents={}",
            summary.riot_client_version, summary.skin_count, summary.weapon_count, summary.agent_count
        );
    } else if let Ok(summary) = engine.get_asset_summary() {
        println!(
            "Assets cached  : clientVersion={}, skins={}, weapons={}, agents={}",
            summary.riot_client_version, summary.skin_count, summary.weapon_count, summary.agent_count
        );
    }

    println!("\n--- 1. Collection & Owned Entitlements ---");
    match engine.fetch_collection_summary().await {
        Ok(col) => {
            let skins = engine.get_owned_skin_uuids().await.len();
            let levels = engine.get_owned_level_uuids().await.len();
            let chromas = engine.get_owned_chroma_uuids().await.len();
            let sprays = engine.get_owned_spray_uuids().await.len();
            let cards = engine.get_owned_card_uuids().await.len();
            let titles = engine.get_owned_title_uuids().await.len();
            let agents = engine.get_owned_agent_uuids().await.len();
            println!(
                "Collection OK  : {} skins (~{} VP / ${:.2}), levels={}, chromas={}, sprays={}, cards={}, titles={}, agents={}",
                col.owned_skin_count, col.estimated_total_vp, col.estimated_usd, levels, chromas, sprays, cards, titles, agents
            );
            let _ = skins;
        }
        Err(e) => println!("Collection ERR : {e}"),
    }

    println!("\n--- 2. Player Loadout (personalization/v3 -> v2) ---");
    match engine.fetch_player_loadout().await {
        Ok(lo) => {
            println!(
                "Loadout OK     : version={}, card='{}', title='{}', incognito={}, hideLevel={}, liveSynced={}",
                lo.version, lo.player_card_name, lo.player_title_text, lo.incognito, lo.hide_account_level, lo.is_live_match_synced
            );
            println!("Sprays/Flex    : {} slots", lo.sprays.len());
            for s in &lo.sprays {
                println!(
                    "  - [{}] slot={} -> '{}' (flex={}, icon={})",
                    s.slot_label,
                    s.slot_uuid,
                    s.spray_name,
                    s.is_flex,
                    s.spray_icon.is_some()
                );
            }
            println!("Weapons ({})  :", lo.guns.len());
            for g in lo.guns.iter().take(12) {
                println!(
                    "  - {:<10} : {} [live={}, next={:?}, buddy={:?}]",
                    g.weapon_name, g.skin_name, g.is_live_match_skin, g.next_match_skin_name, g.buddy_name
                );
            }
        }
        Err(e) => println!("Loadout ERR    : {e}"),
    }

    println!("\n--- 3. Live Match / Party State ---");
    match engine.poll_live_state().await {
        Ok(LiveState::Idle { message, .. }) => println!("Live State     : Idle ({message})"),
        Ok(LiveState::InParty(p)) => println!(
            "Live State     : InParty state='{}' queue='{}' members={}",
            p.state, p.queue_id, p.members.len()
        ),
        Ok(LiveState::PreGame(pg)) => {
            println!(
                "Live State     : PreGame match={} map='{}' queue='{}' allies={} stacks={}",
                pg.match_id,
                pg.map_name,
                pg.queue_id,
                pg.allies.len(),
                pg.detected_stacks.len()
            );
            for s in &pg.detected_stacks {
                println!(
                    "  stack [{}] team={} size={} confidence_matches={} members={:?}",
                    s.label, s.team_id, s.member_puuids.len(), s.confidence_shared_matches, s.member_puuids
                );
            }
            for a in &pg.allies {
                println!(
                    "  ally  {:<24} rank={} ({}RR) dots={:?} party={:?} puuid={}",
                    a.display_name, a.current_rank.tier_name, a.current_rank.rr, a.recent_outcomes, a.party_group_label, a.puuid
                );
            }
        }
        Ok(LiveState::CoreGame(cg)) => {
            println!(
                "Live State     : CoreGame match={} map='{}' queue='{}' server='{}' players={} stacks={}",
                cg.match_id,
                cg.map_name,
                cg.queue_id,
                cg.server_name,
                cg.my_team.len() + cg.enemy_team.len(),
                cg.detected_stacks.len()
            );
            for s in &cg.detected_stacks {
                println!(
                    "  stack [{}] team={} size={} confidence_matches={} members={:?}",
                    s.label, s.team_id, s.member_puuids.len(), s.confidence_shared_matches, s.member_puuids
                );
            }
            for p in cg.my_team.iter().chain(cg.enemy_team.iter()) {
                println!(
                    "  player [{}] {:<22} rank={} ({}RR) dots={:?} party={:?} puuid={}",
                    p.team_id, p.display_name, p.current_rank.tier_name, p.current_rank.rr, p.recent_outcomes, p.party_group_label, p.puuid
                );
            }
        }
        Err(e) => println!("Live State ERR : {e}"),
    }

    println!("\n--- 4. Storefront & Wallet ---");
    match engine.fetch_storefront().await {
        Ok(sf) => {
            let names: Vec<&str> = sf.daily_offers.iter().map(|d| d.skin_name.as_str()).collect();
            println!(
                "Storefront OK  : daily={} {:?}, bundles={}, wallet=(VP={}, RP={}, KC={})",
                sf.daily_offers.len(),
                names,
                sf.featured_bundles.len(),
                sf.wallet.vp,
                sf.wallet.radianite,
                sf.wallet.kingdom_credits
            );
        }
        Err(e) => println!("Storefront ERR : {e}"),
    }

    println!("\n--- 5. Career & Missions ---");
    match engine.fetch_career().await {
        Ok(c) => {
            println!(
                "Career OK      : {} ({} RR), peak={} ({}), penalties={}, missions={}",
                c.current_rank.tier_name,
                c.current_rank.rr,
                c.peak_rank.tier_name,
                c.peak_rank_act_name,
                c.penalties.len(),
                c.missions.len()
            );
            for m in &c.missions {
                println!(
                    "  - {} ({}/{}, +{} XP, done={})",
                    m.title, m.current_progress, m.target_progress, m.xp_grant, m.completed
                );
            }
        }
        Err(e) => println!("Career ERR     : {e}"),
    }

    if test_put {
        println!("\n--- 6. No-Op PUT Verification ---");
        match engine.test_loadout_v3_noop_put().await {
            Ok((api_ver, version, guns, exprs)) => {
                println!(
                    "PUT OK         : api=v{api_ver}, version={version}, guns={guns}, active_expressions={exprs}"
                );
            }
            Err(e) => {
                eprintln!("PUT FAILED     : {e}");
                process::exit(2);
            }
        }
    }

    if explore_extra {
        println!("\n--- 7. Extra 2026 Endpoint Shape Inspection ---");
        let client_ver = engine
            .get_asset_summary()
            .map(|s| s.riot_client_version)
            .unwrap_or_else(|_| "release-10.00-shipping-0-0".to_string());
        if let Ok(headers) = build_riot_headers(&session, &client_ver) {
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(12))
                .build()
                .unwrap();
            let pd = pd_base(&session.shard);

            let extra_paths = [
                ("personalization_v3_raw", format!("{pd}/personalization/v3/players/{}/playerloadout", session.puuid)),
                ("comp_updates_raw", format!("{pd}/mmr/v1/players/{}/competitiveupdates?startIndex=0&endIndex=8&queue=competitive", session.puuid)),
                ("account_xp", format!("{pd}/account-xp/v1/players/{}", session.puuid)),
                ("bloomline_player", format!("{pd}/bloomline/v0/players/{}", session.puuid)),
                ("all_owned_items", format!("{pd}/store/v1/entitlements/{}", session.puuid)),
            ];

            for (label, url) in extra_paths {
                match http.get(&url).headers(headers.clone()).send().await {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let text = resp.text().await.unwrap_or_default();
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            let keys: Vec<String> = val
                                .as_object()
                                .map(|m| m.keys().cloned().collect())
                                .unwrap_or_default();
                            match label {
                                "personalization_v3_raw" => {
                                    let ae_len = val
                                        .get("ActiveExpressions")
                                        .and_then(|v| v.as_array())
                                        .map(|a| a.len())
                                        .unwrap_or(0);
                                    let gun_len = val
                                        .get("Guns")
                                        .and_then(|v| v.as_array())
                                        .map(|a| a.len())
                                        .unwrap_or(0);
                                    let ae_types: Vec<String> = val
                                        .get("ActiveExpressions")
                                        .and_then(|v| v.as_array())
                                        .map(|a| {
                                            a.iter()
                                                .map(|e| {
                                                    format!(
                                                        "{}:{}",
                                                        e.get("TypeID").and_then(|x| x.as_str()).unwrap_or("?"),
                                                        e.get("AssetID").and_then(|x| x.as_str()).unwrap_or("?")
                                                    )
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    println!(
                                        "{label:<22}: HTTP {status}, keys={keys:?}, guns={gun_len}, active_exprs={ae_len} {ae_types:?}"
                                    );
                                }
                                "account_xp" => {
                                    let lvl = val
                                        .get("Progress")
                                        .and_then(|p| p.get("Level"))
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0);
                                    let xp = val
                                        .get("Progress")
                                        .and_then(|p| p.get("XP"))
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0);
                                    println!("{label:<22}: HTTP {status}, keys={keys:?}, level={lvl}, xp={xp}");
                                }
                                "bloomline_player" => {
                                    let bonsais = val
                                        .get("Bonsais")
                                        .and_then(|v| v.as_array())
                                        .map(|a| a.len())
                                        .unwrap_or(0);
                                    println!("{label:<22}: HTTP {status}, keys={keys:?}, bonsais={bonsais}");
                                }
                                "comp_updates_raw" => {
                                    let entries: Vec<String> = val
                                        .get("Matches")
                                        .and_then(|v| v.as_array())
                                        .map(|a| {
                                            a.iter()
                                                .map(|m| {
                                                    let t0 = m.get("TierBeforeUpdate").and_then(|x| x.as_i64()).unwrap_or(0);
                                                    let t1 = m.get("TierAfterUpdate").and_then(|x| x.as_i64()).unwrap_or(0);
                                                    let rr = m.get("RankedRatingEarned").and_then(|x| x.as_i64()).unwrap_or(0);
                                                    let map = m.get("MapID").and_then(|x| x.as_str()).unwrap_or("?");
                                                    let mv = m.get("CompetitiveMovement").and_then(|x| x.as_str()).unwrap_or("?");
                                                    format!("{t0}->{t1}({rr:+})@{map}[{mv}]")
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    println!("{label:<22}: HTTP {status}, keys={keys:?}, matches={entries:?}");
                                }
                                "all_owned_items" => {
                                    let groups: Vec<String> = val
                                        .get("EntitlementsByTypes")
                                        .and_then(|v| v.as_array())
                                        .map(|a| {
                                            a.iter()
                                                .map(|g| {
                                                    let tid = g.get("ItemTypeID").and_then(|x| x.as_str()).unwrap_or("?");
                                                    let cnt = g.get("Entitlements").and_then(|x| x.as_array()).map(|x| x.len()).unwrap_or(0);
                                                    format!("{tid}={cnt}")
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    println!("{label:<22}: HTTP {status}, keys={keys:?}, types={groups:?}");
                                }
                                _ => println!("{label:<22}: HTTP {status}, keys={keys:?}"),
                            }
                        } else {
                            let snip: String = text.chars().take(140).collect();
                            println!("{label:<22}: HTTP {status}, non-json={snip}");
                        }
                    }
                    Err(e) => println!("{label:<22}: ERR {e}"),
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: myval-cli <sync-assets|status|login-url|probe> [--db <path>] [--session <path>] [--redirect-url <url>] [--test-put] [--explore-extra]");
        process::exit(1);
    }
    let cmd = args[1].as_str();
    if cmd == "login-url" {
        println!("{}", build_riot_login_url());
        return;
    }

    let dev_dir = default_dev_dir();
    let mut db_path = if cmd == "probe" {
        dev_dir.join("dev.db")
    } else {
        PathBuf::from("myval.db")
    };
    let mut session_path = dev_dir.join("session.json");
    let mut redirect_url: Option<String> = None;
    let mut test_put = false;
    let mut explore_extra = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--db" if i + 1 < args.len() => {
                db_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--session" if i + 1 < args.len() => {
                session_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--redirect-url" if i + 1 < args.len() => {
                redirect_url = Some(args[i + 1].clone());
                i += 2;
            }
            "--test-put" => {
                test_put = true;
                i += 1;
            }
            "--explore-extra" => {
                explore_extra = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    if cmd == "probe" {
        run_probe(
            &db_path,
            &session_path,
            redirect_url.as_deref(),
            test_put,
            explore_extra,
        )
        .await;
        return;
    }

    let engine = MyValEngineCore::open(db_path.to_str().unwrap_or("myval.db"))
        .expect("failed to open sqlite db");

    match cmd {
        "sync-assets" => {
            let summary = engine.sync_assets().await.expect("asset sync failed");
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        }
        "status" => {
            let summary = engine.get_asset_summary().expect("status failed");
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        }
        other => {
            eprintln!("Unknown command: {other}");
            process::exit(1);
        }
    }
}
