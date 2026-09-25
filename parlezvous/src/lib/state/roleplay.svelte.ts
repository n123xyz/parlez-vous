import { ROLEPLAY_SCENARIOS, type RoleplayPersona } from '$lib/roleplay';

export interface RoleplayState {
    activeScenario: RoleplayPersona | null;
    completedObjectives: Record<string, boolean>;
    isSelectorOpen: boolean;
    isMissionExpanded: boolean;
    isCustomModalOpen: boolean;
    customTitle: string;
    customRole: string;
    customSetting: string;
    customPrompt: string;
    customVrm: 'avatar.vrm' | 'man.vrm';
    customGreeting: string;
}

export const roleplayState = $state<RoleplayState>({
    activeScenario: null,
    completedObjectives: {},
    isSelectorOpen: false,
    isMissionExpanded: true,
    isCustomModalOpen: false,
    customTitle: '',
    customRole: '',
    customSetting: '',
    customPrompt: '',
    customVrm: 'avatar.vrm',
    customGreeting: ''
});

export function selectScenario(scenario: RoleplayPersona | null) {
    roleplayState.activeScenario = scenario;
    roleplayState.completedObjectives = {};
    roleplayState.isSelectorOpen = false;
    roleplayState.isMissionExpanded = true;
}

export function clearScenario() {
    roleplayState.activeScenario = null;
    roleplayState.completedObjectives = {};
}

export function toggleObjective(id: string) {
    roleplayState.completedObjectives[id] = !roleplayState.completedObjectives[id];
}

export function createCustomScenario(): RoleplayPersona {
    const title = roleplayState.customTitle.trim() || 'Custom Roleplay';
    const role = roleplayState.customRole.trim() || 'Roleplay Partner';
    const setting = roleplayState.customSetting.trim() || 'Custom Scenario Setting';
    const prompt = roleplayState.customPrompt.trim() || `You are ${role} in this scenario: ${setting}. Converse naturally with the user.`;
    const greeting = roleplayState.customGreeting.trim() || 'Hello! Welcome. How can I help you today?';

    const customScenario: RoleplayPersona = {
        id: `custom_${Date.now()}`,
        title,
        personaName: role,
        role,
        category: 'custom',
        difficulty: 'A2',
        vrmModel: roleplayState.customVrm,
        icon: '🎭',
        color: 'from-violet-500/20 to-indigo-500/10 border-violet-500/30 text-violet-300',
        badgeColor: 'bg-violet-500/10 text-violet-400 border-violet-500/20',
        setting,
        summary: `Custom situational practice: ${role} - ${setting}`,
        scenarioPrompt: prompt,
        initialGreeting: {
            french: greeting,
            korean: greeting,
            spanish: greeting,
            german: greeting,
            japanese: greeting,
            russian: greeting,
            ukrainian: greeting,
            english: greeting
        },
        initialGreetingNative: greeting,
        suggestedPhrases: [],
        objectives: [
            { id: 'custom_1', description: 'Introduce yourself and begin the scenario' },
            { id: 'custom_2', description: 'Discuss details and maintain immersion' },
            { id: 'custom_3', description: 'Reach a conclusion or farewell' }
        ]
    };

    roleplayState.activeScenario = customScenario;
    roleplayState.completedObjectives = {};
    roleplayState.isCustomModalOpen = false;
    roleplayState.isSelectorOpen = false;
    return customScenario;
}
