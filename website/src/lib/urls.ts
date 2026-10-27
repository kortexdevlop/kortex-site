export function canonical(path:string,origin:string):string {
  const base=new URL(origin);
  if(!['https:','http:'].includes(base.protocol)||base.username||base.password||base.pathname!=='/'||base.search||base.hash)throw Error('Invalid canonical origin');
  if(base.protocol!=='https:'&&!['localhost','127.0.0.1','[::1]'].includes(base.hostname))throw Error('HTTPS required');
  if(!path.startsWith('/')||path.startsWith('//')||path.includes('\\'))throw Error('Invalid canonical path');
  const url=new URL(path,base);if(url.origin!==base.origin)throw Error('Invalid canonical URL');url.hash='';return url.href;
}
