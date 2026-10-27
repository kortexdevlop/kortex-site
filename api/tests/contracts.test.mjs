import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
const root=resolve(import.meta.dirname,'../..');
const read=p=>JSON.parse(readFileSync(resolve(root,p),'utf8'));
const ajv=new Ajv2020({strict:false});addFormats(ajv);
for(const name of ['manifest','contract','artifact','api'])ajv.addSchema(read(`shared/schemas/${name}.schema.json`),`${name}.schema.json`);
const validate=name=>ajv.getSchema(`api.schema.json#/$defs/${name}`);
test('health and errors are closed, bounded API V1 envelopes',()=>{
 const health=validate('Health'),error=validate('Error');
 assert.equal(health({status:'ok',api_version:'1'}),true);assert.equal(health({status:'ok',api_version:'2'}),false);assert.equal(health({status:'ok',api_version:'1',secret:'no'}),false);
 const body={error:{code:'NOT_FOUND',message:'Route not found',request_id:'92000000-0000-4000-8000-000000000001'}};
 assert.equal(error(body),true);assert.equal(error({...body,error:{...body.error,code:'SQL_TRACE'}}),false);assert.equal(error({...body,error:{...body.error,message:'x'.repeat(513)}}),false);
});
test('catalog DTOs use frozen manifest fields and bounded pagination',()=>{
 const m=read('tools/calculatrice/manifest.json'),p=read('tests/fixtures/artifact-signed-test.json');
 const summary=Object.fromEntries(['id','version','name','summary','category','tags','icon_path','abi_version'].map(k=>[k,m[k]]));Object.assign(summary,{sha256:p.sha256,published_at:100});
 assert.equal(validate('ToolSummary')(summary),true);assert.equal(validate('ToolList')({items:[summary],next_cursor:null}),true);
 assert.equal(validate('ToolList')({items:Array(101).fill(summary),next_cursor:null}),false);assert.equal(validate('ToolList')({items:[],next_cursor:'x'.repeat(4097)}),false);
 assert.equal(validate('ToolDetail')({tool:summary,manifest:m,contract:read('tools/calculatrice/contract.json'),descriptor:p}),true);
});
test('publication DTO accepts the public example and rejects malformed/unknown fields',()=>{
 const request=read('shared/fixtures/publication-request.json'),check=validate('PublicationRequest');
 assert.equal(check(request),true);assert.equal(check({...request,secret:'no'}),false);assert.equal(check({...request,request_id:'invalid'}),false);assert.equal(check({...request,r2_key:'../overwrite.tool'}),false);
 assert.equal(check({...request,descriptor:{...request.descriptor,sha256:'not-a-hash'}}),false);
});
