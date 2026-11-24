package com.myval.app.ui.main

import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import com.myval.app.ui.components.TacticalSplashScreen
import org.junit.Before
import org.junit.Rule
import org.junit.Test

/** Instrumented UI test verifying Compose UI boot renders the tactical splash view. */
class MainScreenTest {

  @get:Rule val composeTestRule = createAndroidComposeRule<ComponentActivity>()

  @Before
  fun setup() {
    composeTestRule.setContent { TacticalSplashScreen(statusText = "Connecting...") }
  }

  @Test
  fun splashScreen_rendersBrandAndStatus() {
    composeTestRule.onNodeWithText("MYVAL").assertExists()
    composeTestRule.onNodeWithText("Connecting...").assertExists()
  }
}
