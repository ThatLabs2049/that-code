export interface McpPreset {

  id: string;

  labelKey: string;

  descriptionKey: string;

  command: string;

}



export const MCP_PRESETS: McpPreset[] = [

  {

    id: "filesystem",

    labelKey: "mcpPresetFilesystem",

    descriptionKey: "mcpPresetFilesystemDesc",

    command: "npx -y @modelcontextprotocol/server-filesystem .",

  },

  {

    id: "fetch",

    labelKey: "mcpPresetFetch",

    descriptionKey: "mcpPresetFetchDesc",

    command: "npx -y @modelcontextprotocol/server-fetch",

  },

  {

    id: "memory",

    labelKey: "mcpPresetMemory",

    descriptionKey: "mcpPresetMemoryDesc",

    command: "npx -y @modelcontextprotocol/server-memory",

  },

];

