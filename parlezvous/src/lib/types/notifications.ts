export interface NotificationSettings {
  enabled: boolean;
  srs_reviews: boolean;          // Spaced repetition flashcard review due alerts
  daily_streak_reminder: boolean;// Daily study streak maintenance reminder
  study_break_alerts: boolean;   // Remind to take a break after continuous study sessions
  evening_winddown: boolean;     // Evening vocabulary memory consolidation alert
  milestone_alerts: boolean;     // XP & tier level-up milestones
  sound_enabled: boolean;        // Web Audio melodic notification chime
  frequency_minutes: number;     // Check / cooldown interval (e.g. 60, 120 min)
}

export type ParlezNotificationCategory = keyof NotificationSettings | 'test';

export interface ParlezNotificationOptions {
  title: string;
  body: string;
  category: ParlezNotificationCategory;
  icon?: string;
}
