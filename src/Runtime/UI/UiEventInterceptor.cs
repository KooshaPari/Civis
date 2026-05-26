#nullable enable
using System;
using BepInEx.Logging;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Global UI event interceptor that logs ALL button clicks and pointer events
    /// to help diagnose click routing issues. Attached to a persistent DontDestroyOnLoad object.
    /// </summary>
    public class UiEventInterceptor : MonoBehaviour
    {
        private ManualLogSource? _log;
        private readonly string _sessionId = System.Guid.NewGuid().ToString().Substring(0, 8);
        private long _clickLogCount;

        public void SetLogger(ManualLogSource log)
        {
            _log = log;
        }

        private void Awake()
        {
            LogWarning($"[UiEventInterceptor::{_sessionId}] UiEventInterceptor is disabled and will self-destruct.");
            enabled = false;
            Destroy(this);
        }

        private void Start()
        {
        }

        private void Update()
        {
        }

        private void HookButton(Button btn)
        {
            if (btn == null || btn.gameObject.name.Contains("_intercepted")) return;

            try
            {
                // Add a click listener that logs the button name
                btn.onClick.AddListener(() => OnAnyButtonClicked(btn));
                btn.gameObject.name = btn.gameObject.name + "_intercepted";
            }
            catch (Exception ex)
            {
                LogWarning($"[UiEventInterceptor::{_sessionId}] Failed to hook button '{btn.name}': {ex}");
            }
        }

        private void OnAnyButtonClicked(Button btn)
        {
            _clickLogCount++;
            LogInfo($"[UiEventInterceptor::{_sessionId}] ⚡ BUTTON CLICK #{_clickLogCount} at {System.DateTime.UtcNow:HH:mm:ss.fff} UTC");
            LogInfo($"[UiEventInterceptor::{_sessionId}]   Button name: '{btn.name}'");
            LogInfo($"[UiEventInterceptor::{_sessionId}]   GameObject path: {GetGameObjectPath(btn.gameObject)}");
            LogInfo($"[UiEventInterceptor::{_sessionId}]   Active: {btn.gameObject.activeInHierarchy}, Interactable: {btn.interactable}");
            LogInfo($"[UiEventInterceptor::{_sessionId}]   OnClick listeners: {btn.onClick.GetPersistentEventCount()}");
        }

        private static string GetGameObjectPath(GameObject go)
        {
            string path = go.name;
            Transform current = go.transform.parent;
            while (current != null)
            {
                path = current.name + "/" + path;
                current = current.parent;
            }
            return path;
        }

        private void LogInfo(string message)
        {
            if (_log != null)
                _log.LogInfo(message);
        }

        private void LogWarning(string message)
        {
            if (_log != null)
                _log.LogWarning(message);
        }
    }
}
