export interface Theme {
    id: string;
    name: string;
    description: string;
    subthemes: string[];
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
            { 
                id: 'greetings', 
                name: 'Greetings and Introductions', 
                description: 'Learn to say hello and introduce yourself.',
                subthemes: [
                    "Formal Greetings: Workplace, elders, and professional settings.",
                    "Informal Greetings: Slang, casual check-ins, and peer-to-peer.",
                    "Time of Day: Morning, afternoon, evening, and night specifics.",
                    "Farewells & Sign-offs: Goodbyes, 'see you later,' and written closings.",
                    "Personal Details: Stating your name, age, and where you are from.",
                    "Introducing Others: Presenting a friend or colleague to someone new.",
                    "'How are you?': Asking about well-being and common responses.",
                    "Titles & Honorifics: Mr., Ms., Dr., and culturally specific respect markers.",
                    "Body Language & Etiquette: Handshakes, bowing, and physical proximity.",
                    "Politeness Markers: Please, thank you, you're welcome, and apologies."
                ]
            },
            { 
                id: 'numbers', 
                name: 'Numbers & Currency', 
                description: 'Count and handle basic transactions.',
                subthemes: [
                    "Cardinal Numbers: Counting from 1 to 1,000+.",
                    "Ordinal Numbers: 1st, 2nd, 3rd (useful for dates and ranking).",
                    "Decimals & Fractions: Half, quarter, point-five.",
                    "Basic Math: Add, subtract, multiply, and divide vocabulary.",
                    "Measurements: Weight, length, distance, and volume.",
                    "Asking for Prices: 'How much is this?' and negotiating.",
                    "Handling Transactions: Cash, credit cards, change, and receipts.",
                    "Banking Basics: ATMs, accounts, deposits, and withdrawals.",
                    "Currency Terms: Exchange rates, coins, bills, and local money slang.",
                    "Contact Info: Reading phone numbers, zip codes, and addresses aloud."
                ]
            },
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
            { 
                id: 'family', 
                name: 'Family & Friends', 
                description: 'Discuss relationships and appearance.',
                subthemes: [
                    "Immediate Family: Parents, siblings, and children.",
                    "Extended Family: Grandparents, cousins, aunts, and uncles.",
                    "Relationship Status: Single, married, divorced, dating.",
                    "Physical Appearance: Describing height, hair, eye color, and build.",
                    "Personality Traits: Describing character (funny, shy, outgoing, stubborn).",
                    "Friendship Dynamics: Acquaintances, best friends, and socializing.",
                    "Pets & Animals: Types of common household pets and animal care.",
                    "Life Milestones: Births, weddings, graduations, and funerals.",
                    "Generations: Describing age groups (toddlers, teenagers, elderly).",
                    "Household Roles: Family traditions, dynamics, and responsibilities."
                ]
            },
            { 
                id: 'time', 
                name: 'Days, Time, & Holidays', 
                description: 'Navigate schedules and calendar events.',
                subthemes: [
                    "Days of the Week: Weekdays vs. weekends.",
                    "Months & Seasons: The calendar year and seasonal changes.",
                    "Telling Time: Hours, minutes, 'quarter past,' and 'half past.'",
                    "Parts of the Day: Dawn, morning, noon, dusk, and midnight.",
                    "Time Markers: Yesterday, today, tomorrow, and 'next week.'",
                    "Frequency Adverbs: Always, sometimes, rarely, and never.",
                    "National Holidays: Major cultural and religious celebrations.",
                    "Personal Dates: Birthdays and anniversaries.",
                    "Scheduling: Making, changing, or canceling appointments.",
                    "Punctuality: Concepts of being early, on time, or running late."
                ]
            },
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
            { 
                id: 'food', 
                name: 'Food & Cooking', 
                description: 'Discuss recipes, diets, and restaurant dining.',
                subthemes: [
                    "Ingredients: Fruits, vegetables, meats, and grains.",
                    "Meals of the Day: Breakfast, lunch, dinner, and snacks.",
                    "Cooking Verbs: Boil, fry, bake, chop, and grill.",
                    "Kitchenware: Utensils, pots, pans, and appliances.",
                    "Restaurant Dining: Ordering, asking for the menu, and tipping.",
                    "Taste & Texture: Sweet, sour, spicy, salty, bitter, and crunchy.",
                    "Dietary Needs: Allergies, vegetarian/vegan, and gluten-free.",
                    "Recipes: Reading instructions, portions, and measurements.",
                    "Grocery Shopping: Navigating the supermarket and reading labels.",
                    "Food Culture: Traditional dishes, street food, and dining etiquette."
                ]
            },
            { 
                id: 'house', 
                name: 'Around the House', 
                description: 'Household chores, furniture, and daily routines.',
                subthemes: [
                    "Rooms of the House: Kitchen, bedroom, bathroom, living room.",
                    "Types of Housing: Apartments, houses, dorms, and countryside villas.",
                    "Furniture & Decor: Beds, sofas, tables, rugs, and lighting.",
                    "Household Chores: Sweeping, laundry, washing dishes, and taking out trash.",
                    "Daily Routines: Waking up, getting dressed, and bedtime habits.",
                    "Personal Hygiene: Showers, brushing teeth, and bathroom supplies.",
                    "Home Maintenance: Fixing leaks, changing bulbs, and basic repairs.",
                    "Real Estate: Renting, buying, landlords, and leases.",
                    "Outdoor Spaces: Gardens, balconies, patios, and garages.",
                    "Utilities: Electricity, water, heating, AC, and internet."
                ]
            },
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
            { 
                id: 'city', 
                name: 'City Life & Directions', 
                description: 'Navigate urban environments and public transit.',
                subthemes: [
                    "Public Transit: Buses, trains, subways, and trams.",
                    "Giving Directions: Turn left, go straight, cross the street.",
                    "Prepositions of Place: Next to, behind, across from, and between.",
                    "Urban Landmarks: Banks, post offices, hospitals, and parks.",
                    "Street Infrastructure: Sidewalks, intersections, traffic lights, and bridges.",
                    "Ticketing: Buying passes, checking schedules, and fares.",
                    "Driving & Traffic: Cars, parking, speed limits, and traffic jams.",
                    "Neighborhoods: Downtown, suburbs, residential, and commercial zones.",
                    "Safety & Emergencies: Police, fire department, and reporting theft.",
                    "Map Reading: Understanding cardinal directions (North, South, East, West)."
                ]
            },
            { 
                id: 'hobbies', 
                name: 'Free Time & Hobbies', 
                description: 'Discuss leisure activities and interests.',
                subthemes: [
                    "Sports & Athletics: Soccer, basketball, swimming, and matches.",
                    "Outdoor Recreation: Hiking, camping, cycling, and fishing.",
                    "Indoor Gaming: Board games, card games, and video games.",
                    "Arts & Crafts: Painting, drawing, knitting, and woodworking.",
                    "Music: Playing instruments, singing, and reading sheet music.",
                    "Literature: Reading books, comics, magazines, and writing.",
                    "Fitness: Gym vocabulary, yoga, weightlifting, and stretching.",
                    "Collecting: Stamps, coins, antiques, and memorabilia.",
                    "Gardening: Plant care, growing vegetables, and landscaping.",
                    "Clubs & Groups: Joining local societies, volunteering, and meetups."
                ]
            },
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
            { 
                id: 'career', 
                name: 'Jobs and Careers', 
                description: 'Professional environments and workplace culture.',
                subthemes: [
                    "Common Professions: Teacher, engineer, artist, chef, etc.",
                    "The Job Hunt: Resumes, cover letters, and interviews.",
                    "Workplace Environment: Colleagues, bosses, and corporate hierarchy.",
                    "Office Vocabulary: Desks, meetings, presentations, and supplies.",
                    "Compensation: Salaries, hourly wages, taxes, and benefits.",
                    "Daily Tasks: Managing emails, answering calls, and project deadlines.",
                    "Employment Status: Hiring, firing, quitting, and retiring.",
                    "Entrepreneurship: Starting a business, clients, and freelancing.",
                    "Remote Work: Video calls, digital collaboration, and working from home.",
                    "Work Culture: Professional etiquette, water-cooler talk, and networking."
                ]
            },
            { 
                id: 'health', 
                name: 'Health and the Body', 
                description: 'Medical vocabulary, wellness, and fitness.',
                subthemes: [
                    "Human Anatomy: External body parts and internal organs.",
                    "Common Illnesses: Colds, flus, headaches, and stomachaches.",
                    "Symptoms: Coughing, sneezing, fever, and nausea.",
                    "Medical Care: Visiting the doctor, hospitals, and clinics.",
                    "Pharmacy: Prescriptions, painkillers, vitamins, and bandages.",
                    "First Aid: Treating cuts, burns, sprains, and calling for an ambulance.",
                    "Mental Health: Stress, anxiety, therapy, and self-care.",
                    "Specialists: Dentists, optometrists, surgeons, and dermatologists.",
                    "Healthy Lifestyle: Nutrition, dieting, and sleep hygiene.",
                    "Physical Therapy: Recovery, mobility, and chronic pain management."
                ]
            },
            { 
                id: 'travel', 
                name: 'Travel & Tourism', 
                description: 'Vacations, accommodations, and airports.',
                subthemes: [
                    "Airport Navigation: Check-in, security, gates, and boarding.",
                    "Accommodations: Booking hotels, hostels, and vacation rentals.",
                    "Border Control: Passports, visas, customs, and immigration.",
                    "Luggage: Packing, suitcases, carry-ons, and baggage claim.",
                    "Modes of Travel: Airplanes, cruise ships, long-distance buses.",
                    "Sightseeing: Museums, historical sites, and tour guides.",
                    "Travel Delays: Cancellations, layovers, and lost luggage.",
                    "Souvenirs: Local markets, haggling, and cultural gifts.",
                    "Travel Documents: Tickets, itineraries, and travel insurance.",
                    "Reviews & Recommendations: Writing postcards, reviewing trips, and rating hotels."
                ]
            },
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
            { 
                id: 'arts', 
                name: 'The Arts & Media', 
                description: 'Movies, music, literature, and critique.',
                subthemes: [
                    "Cinema: Movie genres, directors, actors, and screenplays.",
                    "Music Industry: Concerts, albums, genres, and streaming.",
                    "Theater & Dance: Live plays, musicals, ballet, and choreography.",
                    "Visual Arts: Museums, exhibitions, sculpture, and photography.",
                    "Literature & Poetry: Novels, authors, literary devices, and publishing.",
                    "Television: Series, broadcasting, reality TV, and documentaries.",
                    "Journalism: Newspapers, reporters, headlines, and bias.",
                    "Social Media: Influencers, viral content, algorithms, and digital culture.",
                    "Critique & Review: Expressing opinions, analyzing themes, and rating art.",
                    "Cultural Heritage: Folklore, mythology, and historical artifacts."
                ]
            },
            { 
                id: 'technology', 
                name: 'Technology & Science', 
                description: 'Discuss modern advancements and digital life.',
                subthemes: [
                    "Hardware: Computers, smartphones, processors, and screens.",
                    "Software & Apps: Operating systems, coding, and user interfaces.",
                    "The Internet: Browsers, Wi-Fi, cloud storage, and connectivity.",
                    "Cybersecurity: Passwords, hacking, viruses, and data privacy.",
                    "Artificial Intelligence: Machine learning, robotics, and automation.",
                    "Space Exploration: Planets, astronomy, satellites, and rockets.",
                    "Basic Sciences: Physics, chemistry, biology, and the scientific method.",
                    "Research & Labs: Experiments, data analysis, and publishing papers.",
                    "Gadgets & Wearables: Smartwatches, VR, and home automation.",
                    "Future Innovations: Quantum computing, bioengineering, and nanotech."
                ]
            },
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
            { 
                id: 'environment', 
                name: 'The Environment', 
                description: 'Climate change, nature, and conservation.',
                subthemes: [
                    "Climate Change: Global warming, greenhouse gases, and carbon footprints.",
                    "Energy Sources: Renewable (solar/wind) vs. fossil fuels (oil/coal).",
                    "Pollution: Air, water, soil contamination, and smog.",
                    "Waste Management: Recycling, composting, landfills, and plastics.",
                    "Natural Disasters: Earthquakes, hurricanes, wildfires, and floods.",
                    "Ecosystems: Oceans, rainforests, deserts, and biodiversity.",
                    "Wildlife Conservation: Endangered species, poaching, and habitats.",
                    "Agriculture: Factory farming, organic growing, and pesticides.",
                    "Geography: Mountains, rivers, continents, and topography.",
                    "Meteorology: Extreme weather patterns, forecasting, and atmospheric science."
                ]
            },
            { 
                id: 'news', 
                name: 'Current Events', 
                description: 'Global politics, economy, and society.',
                subthemes: [
                    "Global Politics: Elections, governments, democracies, and dictatorships.",
                    "Economics: Inflation, stock markets, trade agreements, and poverty.",
                    "International Relations: Diplomacy, treaties, the UN, and foreign policy.",
                    "Social Justice: Human rights, protests, equality, and systemic issues.",
                    "Public Health: Pandemics, global aid, and healthcare access.",
                    "Law & Justice: Courts, crime, legislation, and the penal system.",
                    "Migration: Refugees, borders, immigration policy, and citizenship.",
                    "Labor Movements: Strikes, unions, workers' rights, and wage gaps.",
                    "Education Policy: Access to schooling, university funding, and literacy.",
                    "Philanthropy: NGOs, charities, disaster relief, and volunteerism."
                ]
            },
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

export function getRandomThemeAndSubthemeForTier(level: number): { theme: string, subtheme: string } {
    const tier = CURRICULUM_TIERS.find(t => t.level === level) || CURRICULUM_TIERS[0];
    const randomTheme = tier.themes[Math.floor(Math.random() * tier.themes.length)];
    const randomSubtheme = randomTheme.subthemes[Math.floor(Math.random() * randomTheme.subthemes.length)];
    return { theme: randomTheme.id, subtheme: randomSubtheme };
}
