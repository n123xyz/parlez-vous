import toast from 'svelte-french-toast';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification
} from '@tauri-apps/plugin-notification';
import type { NotificationSettings, ParlezNotificationOptions } from '$lib/types/notifications';
import { settingsState } from '$lib/state/settings.svelte.ts';
import { timeTracker } from '$lib/state/timeTracker.svelte.ts';
import { invoke } from '@tauri-apps/api/core';

const STORAGE_KEY = 'parlezvous_notifications';

export const defaultNotificationSettings: NotificationSettings = {
  enabled: true,
  srs_reviews: true,
  daily_streak_reminder: true,
  study_break_alerts: true,
  evening_winddown: true,
  milestone_alerts: true,
  sound_enabled: true,
  frequency_minutes: 60,
};

export const notificationState = $state<{
  settings: NotificationSettings;
  permission: NotificationPermission | 'unsupported';
  isInitialized: boolean;
}>({
  settings: { ...defaultNotificationSettings },
  permission: 'default',
  isInitialized: false,
});

// Cooldown tracker to prevent repetitive notification spamming (timestamps in ms)
const lastAlertTimestamps: Record<string, number> = {};

/**
 * Play a light, pleasant 3-tone harmonic chime via Web Audio (Zero memory / zero external files)
 */
export function playParlezChime(): void {
  if (!notificationState.settings.sound_enabled) return;
  try {
    const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
    if (!AudioCtx) return;
    const ctx = new AudioCtx();
    const now = ctx.currentTime;

    // Pleasant melodic arpeggio (C5: 523.25 Hz -> E5: 659.25 Hz -> G5: 783.99 Hz)
    const freqs = [523.25, 659.25, 783.99];
    freqs.forEach((freq, i) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();

      osc.type = 'sine';
      osc.frequency.setValueAtTime(freq, now + i * 0.1);

      gain.gain.setValueAtTime(0.15, now + i * 0.1);
      gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.1 + 0.35);

      osc.connect(gain);
      gain.connect(ctx.destination);

      osc.start(now + i * 0.1);
      osc.stop(now + i * 0.1 + 0.35);
    });
  } catch (err) {
    console.warn('[Notifications] Audio chime unavailable:', err);
  }
}

/**
 * Load notification settings from localStorage and check OS permission
 */
export function loadNotificationSettings(): NotificationSettings {
  if (typeof window === 'undefined') return defaultNotificationSettings;

  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      notificationState.settings = { ...defaultNotificationSettings, ...parsed };
    }
  } catch (e) {
    console.warn('Failed to load notification settings:', e);
  }

  isPermissionGranted()
    .then((granted) => {
      notificationState.permission = granted ? 'granted' : 'default';
    })
    .catch(() => {
      try {
        if (typeof window !== 'undefined' && 'Notification' in window && typeof (window as any).Notification === 'function') {
          notificationState.permission = (window as any).Notification?.permission || 'default';
        } else {
          notificationState.permission = 'default';
        }
      } catch {
        notificationState.permission = 'default';
      }
    });

  notificationState.isInitialized = true;
  return notificationState.settings;
}

/**
 * Save notification settings to localStorage
 */
export function saveNotificationSettings(newSettings: NotificationSettings): void {
  notificationState.settings = { ...newSettings };
  if (typeof window !== 'undefined') {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(notificationState.settings));
      toast.success('Notification preferences saved!');
    } catch (e) {
      console.warn('Failed to save notification settings:', e);
    }
  }
}

/**
 * Request native OS Notification permission from user
 */
export async function requestNotificationPermission(): Promise<boolean> {
  try {
    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }
    notificationState.permission = permissionGranted ? 'granted' : 'denied';
    if (permissionGranted) {
      toast.success('System notifications enabled for ParlezVous!');
      return true;
    } else {
      toast.error('Notification permission was denied or dismissed');
      return false;
    }
  } catch (e) {
    console.error('Error requesting notification permission via Tauri plugin:', e);
    try {
      if (typeof window !== 'undefined' && 'Notification' in window && typeof (window as any).Notification === 'function') {
        const perm = await (window as any).Notification.requestPermission();
        notificationState.permission = perm;
        if (perm === 'granted') {
          toast.success('System notifications enabled for ParlezVous!');
          return true;
        }
      }
    } catch {}
    toast.error('Notification permission could not be granted');
    return false;
  }
}

/**
 * Send a notification (OS notification + in-app toast + audio chime)
 * Pure deterministic rule-based, zero LLM / zero model memory footprint.
 */
export function sendParlezNotification(options: ParlezNotificationOptions): boolean {
  if (!notificationState.settings.enabled) return false;

  // Verify specific category is enabled (unless test)
  if (options.category !== 'test') {
    const isCategoryEnabled = notificationState.settings[options.category];
    if (isCategoryEnabled === false) return false;
  }

  // 1. Play melodic notification chime
  playParlezChime();

  // 2. Dispatch in-app toast styled for ParlezVous
  toast(options.body, {
    icon: options.icon || '💬',
    duration: 6000,
    style: 'border: 1px solid rgba(253, 253, 150, 0.4); background: #09090b; color: #f4f4f5; font-size: 13px; font-weight: 600;',
  });

  // 3. Dispatch Native OS Notification via Tauri Notification Plugin
  try {
    sendNotification({
      title: options.title,
      body: options.body,
    });
  } catch (err) {
    console.warn('Tauri native notification dispatch error:', err);
    try {
      if (typeof window !== 'undefined' && 'Notification' in window && typeof (window as any).Notification === 'function') {
        if ((window as any).Notification?.permission === 'granted') {
          new (window as any).Notification(options.title, {
            body: options.body,
            icon: '/icons/128x128.png',
            tag: `parlezvous-${options.category}`,
          });
        }
      }
    } catch (fallbackErr) {
      console.warn('Web notification dispatch fallback error:', fallbackErr);
    }
  }

  return true;
}

/**
 * Evaluates language learning state against deterministic trigger rules.
 * Uses cooldown timers to prevent repeating alerts within the user's frequency window.
 * 100% Zero-AI / Zero-Inference.
 */
export async function evaluateStudyNotifications(): Promise<void> {
  if (!notificationState.settings.enabled) return;

  const now = Date.now();
  const cooldownMs = (notificationState.settings.frequency_minutes || 60) * 60 * 1000;
  const targetLanguage = settingsState.targetLanguage || 'Target Language';

  // 1. Spaced Repetition (SRS) Flashcards Due Alert
  if (notificationState.settings.srs_reviews) {
    const last = lastAlertTimestamps['srs_reviews'] || 0;
    if (now - last > Math.max(cooldownMs, 2 * 60 * 60 * 1000)) { // Minimum 2h cooldown
      try {
        const vocab = await invoke<any[]>('get_all_vocabulary').catch(() => []);
        if (vocab && vocab.length > 0) {
          lastAlertTimestamps['srs_reviews'] = now;
          sendParlezNotification({
            title: `ParlezVous: Flashcards Ready!`,
            body: `📚 You have ${vocab.length} vocabulary words ready for review in ${targetLanguage}! Keep your memory fresh!`,
            category: 'srs_reviews',
            icon: '📚',
          });
          return;
        }
      } catch (e) {
        // Silent catch for background evaluation
      }
    }
  }

  // 2. Daily Study Streak Reminder (Triggered in afternoon/evening if active time is 0)
  if (notificationState.settings.daily_streak_reminder) {
    const currentHour = new Date().getHours();
    // Afternoon reminder window (between 2:00 PM and 7:00 PM)
    if (currentHour >= 14 && currentHour <= 19) {
      const last = lastAlertTimestamps['daily_streak'] || 0;
      if (now - last > 18 * 60 * 60 * 1000) { // Once per day (18h cooldown)
        if (timeTracker.activeSeconds < 120) {
          lastAlertTimestamps['daily_streak'] = now;
          sendParlezNotification({
            title: `ParlezVous: Daily Practice Reminder!`,
            body: `🔥 Keep your study streak alive! Spend 5 minutes practicing ${targetLanguage} or completing a journal entry today!`,
            category: 'daily_streak_reminder',
            icon: '🔥',
          });
          return;
        }
      }
    }
  }

  // 3. Continuous Study Session Break Alert (Every 30 minutes of continuous active study)
  if (notificationState.settings.study_break_alerts) {
    if (timeTracker.activeSeconds >= 1800) { // 30 minutes active
      const last = lastAlertTimestamps['study_break'] || 0;
      if (now - last > 30 * 60 * 1000) { // 30 min cooldown
        lastAlertTimestamps['study_break'] = now;
        sendParlezNotification({
          title: `ParlezVous: Time for a Quick Break!`,
          body: `☕ Great focus! You've been actively learning ${targetLanguage} for 30 minutes. Rest your eyes, stretch, and grab some water!`,
          category: 'study_break_alerts',
          icon: '☕',
        });
        return;
      }
    }
  }

  // 4. Evening Memory Consolidation Window (9:00 PM - 10:30 PM)
  if (notificationState.settings.evening_winddown) {
    const currentHour = new Date().getHours();
    const currentMinute = new Date().getMinutes();
    if (currentHour === 21 || (currentHour === 22 && currentMinute <= 30)) {
      const last = lastAlertTimestamps['evening_consolidation'] || 0;
      if (now - last > 18 * 60 * 60 * 1000) { // Once per night
        lastAlertTimestamps['evening_consolidation'] = now;
        sendParlezNotification({
          title: `ParlezVous: Evening Consolidation!`,
          body: `🌙 Review today's ${targetLanguage} vocabulary before bed to maximize overnight memory consolidation!`,
          category: 'evening_winddown',
          icon: '🌙',
        });
        return;
      }
    }
  }
}

/**
 * Send an immediate test notification to verify OS notification, toast, and audio chime
 */
export function sendTestNotification(): void {
  const targetLanguage = settingsState.targetLanguage || 'French';
  sendParlezNotification({
    title: `ParlezVous: Notification System Active!`,
    body: `✨ System notifications, in-app alerts, and audio chimes are fully operational for your ${targetLanguage} studies!`,
    category: 'test',
    icon: '✨',
  });
}

let tickerInterval: any = null;

/**
 * Start the background notification rule evaluator ticker (runs every 60s, completely zero AI / low CPU)
 */
export function startNotificationTicker(): void {
  if (typeof window === 'undefined') return;
  loadNotificationSettings();

  if (tickerInterval) clearInterval(tickerInterval);

  tickerInterval = setInterval(() => {
    try {
      evaluateStudyNotifications();
    } catch (e) {
      console.warn('[Notifications] Error in ticker evaluation:', e);
    }
  }, 60 * 1000); // Check every 60 seconds
}
