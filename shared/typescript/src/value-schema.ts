// Frozen V1 JSON Schema subset, interpreted without eval or dynamic compilation.
export function matchesValueSchema(schema:Record<string,unknown>,value:unknown):boolean{
 let nodes=0;
 function visit(s:Record<string,unknown>,v:unknown,depth:number):boolean{
  if(++nodes>2000||depth>16)return false;
  if(Object.keys(s).some(k=>!['type','properties','required','additionalProperties','items','enum','minimum','maximum','minLength','maxLength','minItems','maxItems'].includes(k)))return false;
  if(Array.isArray(s.enum)&&!s.enum.some(x=>x===v))return false;
  switch(s.type){
   case 'null':return v===null;
   case 'boolean':return typeof v==='boolean';
   case 'number':case 'integer':return typeof v==='number'&&Number.isFinite(v)&&Math.abs(v)<=Number.MAX_SAFE_INTEGER&&(s.type!=='integer'||Number.isSafeInteger(v))&&(s.minimum===undefined||v>=Number(s.minimum))&&(s.maximum===undefined||v<=Number(s.maximum));
   case 'string':return typeof v==='string'&&[...v].length>=Number(s.minLength??0)&&[...v].length<=Number(s.maxLength??32768);
   case 'array':return Array.isArray(v)&&v.length>=Number(s.minItems??0)&&v.length<=Number(s.maxItems??1000)&&v.every(x=>s.items&&visit(s.items as Record<string,unknown>,x,depth+1));
   case 'object':{
    if(!v||typeof v!=='object'||Array.isArray(v))return false;const object=v as Record<string,unknown>,properties=(s.properties??{}) as Record<string,Record<string,unknown>>;
    if(Object.keys(object).length>100||Array.isArray(s.required)&&s.required.some(k=>typeof k!=='string'||!Object.hasOwn(object,k)))return false;
    return Object.entries(object).every(([key,item])=>Object.hasOwn(properties,key)?visit(properties[key],item,depth+1):s.additionalProperties!==false);
   }
   default:return false;
  }
 }
 return visit(schema,value,0);
}
