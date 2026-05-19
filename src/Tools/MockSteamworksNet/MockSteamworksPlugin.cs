#nullable enable
using System;
using BepInEx;
using BepInEx.Logging;
using HarmonyLib;
using Steamworks;

namespace DINOForge.Tools.MockSteamworksNet
{
    /// <summary>
    /// BepInEx plugin that mocks Steamworks.NET managed surface for headless CI testing.
    /// This plugin uses Harmony patches to intercept Steamworks.NET method calls and return
    /// mock responses without requiring a real Steam client or Goldberg emulator.
    ///
    /// IMPORTANT: This plugin is FOR HEADLESS CI TESTING ONLY. It should NOT be deployed
    /// to production or user instances. To use in CI:
    /// 1. Deploy this DLL to BepInEx/plugins/
    /// 2. Launch the game in headless mode (hidden desktop or -nographics)
    /// 3. The plugin automatically patches SteamAPI and returns mock values
    ///
    /// Mocked methods:
    /// - SteamAPI.Init() -> true
    /// - SteamAPI.IsSteamRunning() -> true
    /// - SteamUser.GetSteamID() -> mock CSteamID
    /// - SteamApps.BIsSubscribedApp() -> true
    /// - SteamFriends.GetPersonaName() -> "MockUser"
    /// </summary>
    [BepInPlugin(MockSteamworksPluginInfo.GUID, MockSteamworksPluginInfo.NAME, MockSteamworksPluginInfo.VERSION)]
    public class MockSteamworksPlugin : BaseUnityPlugin
    {
        private static ManualLogSource Log = null!;
        private Harmony? _harmony;

        private void Awake()
        {
            Log = Logger;
            Log.LogInfo($"MockSteamworksNet v{MockSteamworksPluginInfo.VERSION} loading (headless CI mock mode)...");

            try
            {
                _harmony = new Harmony(MockSteamworksPluginInfo.GUID);

                // Patch SteamAPI.Init() to return true without requiring Steam
                var steamApiInitMethod = typeof(SteamAPI).GetMethod("Init", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
                if (steamApiInitMethod != null)
                {
                    var postfixMethod = typeof(MockSteamworksPlugin).GetMethod(nameof(SteamAPI_Init_Postfix), System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                    _harmony.Patch(steamApiInitMethod, postfix: new HarmonyMethod(postfixMethod));
                    Log.LogInfo("Patched SteamAPI.Init()");
                }
                else
                {
                    Log.LogWarning("Could not find SteamAPI.Init() method");
                }

                // Patch SteamAPI.IsSteamRunning() to return true
                var steamApiIsSteamRunningMethod = typeof(SteamAPI).GetMethod("IsSteamRunning", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
                if (steamApiIsSteamRunningMethod != null)
                {
                    var postfixMethod = typeof(MockSteamworksPlugin).GetMethod(nameof(SteamAPI_IsSteamRunning_Postfix), System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                    _harmony.Patch(steamApiIsSteamRunningMethod, postfix: new HarmonyMethod(postfixMethod));
                    Log.LogInfo("Patched SteamAPI.IsSteamRunning()");
                }
                else
                {
                    Log.LogWarning("Could not find SteamAPI.IsSteamRunning() method");
                }

                // Patch SteamUser.GetSteamID() to return a mock CSteamID
                var steamUserGetSteamIdMethod = typeof(SteamUser).GetMethod("GetSteamID", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
                if (steamUserGetSteamIdMethod != null)
                {
                    var postfixMethod = typeof(MockSteamworksPlugin).GetMethod(nameof(SteamUser_GetSteamID_Postfix), System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                    _harmony.Patch(steamUserGetSteamIdMethod, postfix: new HarmonyMethod(postfixMethod));
                    Log.LogInfo("Patched SteamUser.GetSteamID()");
                }
                else
                {
                    Log.LogWarning("Could not find SteamUser.GetSteamID() method");
                }

                // Patch SteamApps.BIsSubscribedApp() to return true
                var steamAppsSubscribedMethod = typeof(SteamApps).GetMethod("BIsSubscribedApp", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static, null, new[] { typeof(uint) }, null);
                if (steamAppsSubscribedMethod != null)
                {
                    var postfixMethod = typeof(MockSteamworksPlugin).GetMethod(nameof(SteamApps_BIsSubscribedApp_Postfix), System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                    _harmony.Patch(steamAppsSubscribedMethod, postfix: new HarmonyMethod(postfixMethod));
                    Log.LogInfo("Patched SteamApps.BIsSubscribedApp()");
                }
                else
                {
                    Log.LogWarning("Could not find SteamApps.BIsSubscribedApp() method");
                }

                // Patch SteamFriends.GetPersonaName() to return mock name
                var steamFriendsPersonaNameMethod = typeof(SteamFriends).GetMethod("GetPersonaName", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
                if (steamFriendsPersonaNameMethod != null)
                {
                    var postfixMethod = typeof(MockSteamworksPlugin).GetMethod(nameof(SteamFriends_GetPersonaName_Postfix), System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                    _harmony.Patch(steamFriendsPersonaNameMethod, postfix: new HarmonyMethod(postfixMethod));
                    Log.LogInfo("Patched SteamFriends.GetPersonaName()");
                }
                else
                {
                    Log.LogWarning("Could not find SteamFriends.GetPersonaName() method");
                }

                Log.LogInfo("MockSteamworksNet patches applied successfully. Headless CI mode active.");
            }
            catch (Exception ex)
            {
                Log.LogError($"Failed to apply Steamworks.NET mocks: {ex}");
            }
        }

        private void OnDestroy()
        {
            if (_harmony != null)
            {
                _harmony.UnpatchSelf();
                Log.LogInfo("MockSteamworksNet patches removed.");
            }
        }

        // Harmony postfix: intercepts SteamAPI.Init() and forces return value to true
        private static void SteamAPI_Init_Postfix(ref bool __result)
        {
            __result = true;
        }

        // Harmony postfix: intercepts SteamAPI.IsSteamRunning() and forces return value to true
        private static void SteamAPI_IsSteamRunning_Postfix(ref bool __result)
        {
            __result = true;
        }

        // Harmony postfix: intercepts SteamUser.GetSteamID() and returns a mock CSteamID
        private static void SteamUser_GetSteamID_Postfix(ref CSteamID __result)
        {
            // Return a mock Steam ID: 76561197960265728 is a well-known test/mock Steam ID
            // This represents the "anonymous" user in Steam conventions
            __result = new CSteamID(76561197960265728UL);
        }

        // Harmony postfix: intercepts SteamApps.BIsSubscribedApp(uint appId) and returns true
        private static void SteamApps_BIsSubscribedApp_Postfix(ref bool __result)
        {
            __result = true;
        }

        // Harmony postfix: intercepts SteamFriends.GetPersonaName() and returns a mock username
        private static void SteamFriends_GetPersonaName_Postfix(ref string __result)
        {
            __result = "MockUser";
        }
    }
}
