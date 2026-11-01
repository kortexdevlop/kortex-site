import type {APIRoute} from 'astro';
import {installScript} from '../lib/desktop-release';
export const prerender=false;
export const GET:APIRoute=()=>new Response(installScript(),{headers:{'Content-Type':'text/plain; charset=utf-8','Cache-Control':'public, max-age=300'}});
