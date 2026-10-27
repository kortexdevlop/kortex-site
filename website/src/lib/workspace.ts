import type {ToolSummary} from './tools';
export type Category='image'|'documents'|'developer'|'media'|'security'|'design'|'other';
export interface WorkspaceTool {tool:ToolSummary;category:Category;glyph:string;mode:'web'|'both'|'local';favorite?:boolean;installed?:boolean;}
export interface RecentItem {name:string;subtitle:string;glyph:string;category:Category;}
export const categories:Array<{id:Category;label:string}>=[{id:'image',label:'Image'},{id:'documents',label:'Documents'},{id:'developer',label:'Développeur'},{id:'media',label:'Vidéo & Audio'},{id:'security',label:'Sécurité'},{id:'design',label:'Design'}];
export function workspaceTools(tools:ToolSummary[]):WorkspaceTool[]{return tools.map(tool=>({tool,category:categories.some(c=>c.id===tool.category)?tool.category as Category:'other',glyph:tool.name.split(/[\s’-]+/).slice(0,2).map(w=>w[0]).join('').toLocaleUpperCase('fr'),mode:'web'}));}
