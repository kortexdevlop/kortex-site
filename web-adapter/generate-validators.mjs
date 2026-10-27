import Ajv from 'ajv/dist/2020.js';
import standalone from 'ajv/dist/standalone/index.js';
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const here=import.meta.dirname,ajv=new Ajv({strict:true,allowUnionTypes:true,code:{source:true,esm:true}});
const ids={};for(const name of ['manifest','contract','ui','workflow','protocol','artifact']){const schema=JSON.parse(readFileSync(resolve(here,`../shared/schemas/${name}.schema.json`)));const id=`urn:kortex:${name}`;ajv.addSchema({...schema,$id:id});ids[name]=id;}
for(const name of ['request','reply']){const id=`urn:kortex:${name}`;ajv.addSchema({$id:id,$ref:`urn:kortex:protocol#/$defs/${name}`});ids[name]=id;}
writeFileSync(resolve(here,'src/generated-validators.ts'),'// @ts-nocheck\n'+standalone.default(ajv,ids));
