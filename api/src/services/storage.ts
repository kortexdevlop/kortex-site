export const MAX_TOOL_BYTES=25*1024*1024;
export class StorageIntegrityError extends Error {}
export interface StoredArtifact {r2_key:string;byte_length:number;head_etag:string;sha256:string}
export async function checkedHead(bucket:R2Bucket,artifact:StoredArtifact):Promise<R2Object|null> {
 const object=await bucket.head(artifact.r2_key);if(!object)return null;
 if(artifact.byte_length<1||artifact.byte_length>MAX_TOOL_BYTES||object.size!==artifact.byte_length||object.etag!==artifact.head_etag)throw new StorageIntegrityError();
 return object;
}
export async function checkedBody(bucket:R2Bucket,artifact:StoredArtifact):Promise<ReadableStream<Uint8Array>|null> {
 const object=await bucket.get(artifact.r2_key,{onlyIf:{etagMatches:artifact.head_etag}});
 if(!object)return null;if(!('body' in object)||object.etag!==artifact.head_etag||object.size!==artifact.byte_length)throw new StorageIntegrityError();
 let count=0;
 return object.body.pipeThrough(new TransformStream<Uint8Array,Uint8Array>({transform(chunk,controller){count+=chunk.byteLength;if(count>artifact.byte_length||count>MAX_TOOL_BYTES)throw new StorageIntegrityError();controller.enqueue(chunk)},flush(){if(count!==artifact.byte_length)throw new StorageIntegrityError()}}));
}
