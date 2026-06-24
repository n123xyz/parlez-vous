export interface Theme {
    id: string;
    name: string;
    description: string;
}

export interface Tier {
    level: number;
    cefr: string;
    name: string;
    description: string;
    color: string;
    xpThreshold: number; // XP needed to unlock this tier
    themes: Theme[];
}

export const XP_THRESHOLDS = {
    1: 0,
    2: 36000,
    3: 144000, // 36k + 108k
    4: 360000, // 144k + 216k
    5: 720000,
    6: 1440000,
    7: 2880000,
};

export const CURRICULUM_TIERS: Tier[] = [
    {
        level: 1,
        cefr: 'A1',
        name: 'The Outskirts',
        description: 'Survival Phrases and Basics.',
        color: 'bg-green-500',
        xpThreshold: XP_THRESHOLDS[1],
        themes: [
            { id: 'greetings', name: 'Greetings and Introductions', description: 'Learn to say hello and introduce yourself.' },
            { id: 'numbers', name: 'Numbers & Currency', description: 'Count and handle basic transactions.' },
        ]
    },
    {
        level: 2,
        cefr: 'A2',
        name: 'The Marketplace',
        description: 'Shopping, Food, and Routine.',
        color: 'bg-emerald-500',
        xpThreshold: XP_THRESHOLDS[2],
        themes: [
            { id: 'family', name: 'Family & Friends', description: 'Discuss relationships and appearance.' },
            { id: 'time', name: 'Days, Time, & Holidays', description: 'Navigate schedules and calendar events.' },
        ]
    },
    {
        level: 3,
        cefr: 'B1',
        name: 'The Town Square',
        description: 'Living Arrangements & Chores.',
        color: 'bg-teal-500',
        xpThreshold: XP_THRESHOLDS[3],
        themes: [
            { id: 'food', name: 'Food & Cooking', description: 'Discuss recipes, diets, and restaurant dining.' },
            { id: 'house', name: 'Around the House', description: 'Household chores, furniture, and daily routines.' },
        ]
    },
    {
        level: 4,
        cefr: 'B2',
        name: 'The Library',
        description: 'City Life & Hobbies.',
        color: 'bg-cyan-500',
        xpThreshold: XP_THRESHOLDS[4],
        themes: [
            { id: 'city', name: 'City Life & Directions', description: 'Navigate urban environments and public transit.' },
            { id: 'hobbies', name: 'Free Time & Hobbies', description: 'Discuss leisure activities and interests.' },
        ]
    },
    {
        level: 5,
        cefr: 'C1',
        name: 'The University',
        description: 'Careers, Health, & Travel.',
        color: 'bg-blue-500',
        xpThreshold: XP_THRESHOLDS[5],
        themes: [
            { id: 'career', name: 'Jobs and Careers', description: 'Professional environments and workplace culture.' },
            { id: 'health', name: 'Health and the Body', description: 'Medical vocabulary, wellness, and fitness.' },
            { id: 'travel', name: 'Travel & Tourism', description: 'Vacations, accommodations, and airports.' },
        ]
    },
    {
        level: 6,
        cefr: 'C2',
        name: 'The Capital',
        description: 'Arts, Story-Telling, Technology.',
        color: 'bg-indigo-500',
        xpThreshold: XP_THRESHOLDS[6],
        themes: [
            { id: 'arts', name: 'The Arts & Media', description: 'Movies, music, literature, and critique.' },
            { id: 'technology', name: 'Technology & Science', description: 'Discuss modern advancements and digital life.' },
        ]
    },
    {
        level: 7,
        cefr: 'Mastery',
        name: 'The Horizon',
        description: 'Environment & Current Events.',
        color: 'bg-purple-500',
        xpThreshold: XP_THRESHOLDS[7],
        themes: [
            { id: 'environment', name: 'The Environment', description: 'Climate change, nature, and conservation.' },
            { id: 'news', name: 'Current Events', description: 'Global politics, economy, and society.' },
        ]
    }
];

export function getTierFromXP(xp: number): number {
    let currentLevel = 1;
    for (let i = 1; i <= 7; i++) {
        if (xp >= XP_THRESHOLDS[i as keyof typeof XP_THRESHOLDS]) {
            currentLevel = i;
        } else {
            break;
        }
    }
    return currentLevel;
}

export function getThemeById(themeId: string): Theme | undefined {
    for (const tier of CURRICULUM_TIERS) {
        const theme = tier.themes.find(t => t.id === themeId);
        if (theme) return theme;
    }
    return undefined;
}
