import assert from 'node:assert/strict';
import test from 'node:test';
import {attachmentKind,checkedAnswer,downloads,fromBase64,resolveStep,toBase64} from '../src/lib/ai-chain.ts';
const step=(bind:object,input={})=>({tool_id:'calculateur-hash',action_id:'sha256_bytes',input,bind,requires_consent:true}) as never;
test('a chain step takes its data from the local file or the previous output only',()=>{
 const file=new TextEncoder().encode('Été');
 assert.equal(resolveStep(step({data_base64:{file:0}}),[file],[]).input.data_base64,toBase64(file));
 assert.equal(resolveStep(step({data_text:{file:0}}),[file],[]).input.data_text,'Été');
 assert.equal(resolveStep(step({data_base64:{step:0,output:'data_base64'}}),[],[{data_base64:'abc'}]).input.data_base64,'abc');
 assert.deepEqual(resolveStep(step({a:{step:0,output:'result'}},{b:10}),[],[{result:1200}]).input,{a:1200,b:10});
 assert.throws(()=>resolveStep(step({data_base64:{file:1}}),[file],[]));
 assert.throws(()=>resolveStep(step({a:{step:0,output:'absent'}}),[],[{result:1}]));
});
test('base64 survives files larger than one conversion chunk',()=>{const bytes=Uint8Array.from({length:100000},(_,i)=>i%251);assert.deepEqual(fromBase64(toBase64(bytes)),bytes)});
test('only the kind of an attachment is ever described',()=>{assert.equal(attachmentKind({type:'image/png',name:'secret.png'}),'image');assert.equal(attachmentKind({type:'',name:'a.PDF'}),'pdf');assert.equal(attachmentKind({type:'',name:'a.csv'}),'text');assert.equal(attachmentKind({type:'',name:'a.bin'}),'other')});
test('a malformed answer is refused and binary outputs become downloads',()=>{assert.throws(()=>checkedAnswer({v:1,reply:'x',steps:[]}));assert.throws(()=>checkedAnswer({v:2,reply:'x',steps:[{tool_id:'A B'}]}));assert.equal(checkedAnswer({v:2,reply:'x',steps:[]}).reply,'x');assert.deepEqual(downloads({data_base64:'abc'},{format:'jpeg'},'compresseur-images'),[{name:'compresseur-images.jpg',type:'image/jpeg',value:'abc'}])});
