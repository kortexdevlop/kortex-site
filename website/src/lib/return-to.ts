export function accountReturn(value:string|null):string{return value&&/^\/account\/connect-desktop\?request_id=[0-9a-f]{8}-[0-9a-f-]{27}$/.test(value)?value:'/account'}
