export async function getThemes(): Promise<Record<string, string[]>> {
    const response = await fetch('/coding/themes.json');
    if (!response.ok) {
        throw new Error('Failed to fetch coding themes');
    }
    return response.json();
}

export function getRandomTheme(themes: Record<string, string[]>, category: string = "All"): string {
    const categories = Object.keys(themes);
    if (categories.length === 0) return "Basic Variables";

    let selectedCategory = category;
    if (selectedCategory === "All" || !themes[selectedCategory]) {
        selectedCategory = categories[Math.floor(Math.random() * categories.length)];
    }

    const topics = themes[selectedCategory];
    if (!topics || topics.length === 0) return "Basic Variables";

    return topics[Math.floor(Math.random() * topics.length)];
}
