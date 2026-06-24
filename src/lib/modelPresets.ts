import type { AgentTier, UpdateAiSettings } from "./settings";



export type ModelPresetId = "balanced" | "fast_plan" | "strong_edit";



export interface ModelPreset {

  id: ModelPresetId;

  labelKey: string;

  descriptionKey: string;

  patch: UpdateAiSettings;

}



export const MODEL_PRESETS: ModelPreset[] = [

  {

    id: "balanced",

    labelKey: "modelPresetBalanced",

    descriptionKey: "modelPresetBalancedDesc",

    patch: {

      defaultAgentTier: "auto",

      autoEscalate: true,

      planBeforeEdit: false,

    },

  },

  {

    id: "fast_plan",

    labelKey: "modelPresetFastPlan",

    descriptionKey: "modelPresetFastPlanDesc",

    patch: {

      defaultAgentTier: "auto",

      autoEscalate: true,

      planBeforeEdit: true,

    },

  },

  {

    id: "strong_edit",

    labelKey: "modelPresetStrongEdit",

    descriptionKey: "modelPresetStrongEditDesc",

    patch: {

      defaultAgentTier: "deep",

      autoEscalate: false,

      planBeforeEdit: false,

    },

  },

];



export function detectActivePreset(settings: {

  defaultAgentTier: AgentTier;

  autoEscalate: boolean;

  planBeforeEdit: boolean;

}): ModelPresetId | null {

  for (const preset of MODEL_PRESETS) {

    const p = preset.patch;

    if (

      p.defaultAgentTier === settings.defaultAgentTier &&

      p.autoEscalate === settings.autoEscalate &&

      p.planBeforeEdit === settings.planBeforeEdit

    ) {

      return preset.id;

    }

  }

  return null;

}

