export const prerender=true;
export function GET(){return Response.json({service:'kortex-web',status:'ok'},{headers:{'Cache-Control':'no-store'}})}
