import {icon} from '../../lib/icons';
export default function ToolTile({category,size=18}:{category:string;size?:number}){
 return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={icon(category)}/></svg>;
}
