export {};
// Home page motion: scroll progress for the scenes, reveals, the prism steps, card tilt and the tool counter.
const reduce=matchMedia('(prefers-reduced-motion: reduce)');
const root=document.documentElement;
const scenes=[...document.querySelectorAll<HTMLElement>('[data-scene]')];
const story=document.querySelector<HTMLElement>('[data-scene=story]');
const hero=document.querySelector<HTMLElement>('[data-scene=hero]');
const stage=hero?.querySelector<HTMLElement>('.lx-stage');
const clamp=(v:number)=>Math.min(1,Math.max(0,v));
// Holds on each face, then turns: the middle half of every segment does the rotation.
const settle=(f:number)=>{const t=clamp((f-.3)/.4);return t*t*(3-2*t);};

function measure(){if(stage&&hero)hero.style.setProperty('--lift',Math.max(0,stage.offsetTop-88)+'px');}
function frame(){
  const vh=innerHeight;
  document.body.classList.toggle('at-top',scrollY<8);
  for(const scene of scenes){
    const r=scene.getBoundingClientRect();
    scene.style.setProperty('--p',clamp(-r.top/Math.max(1,r.height-vh)).toFixed(4));
    scene.style.setProperty('--e',clamp((vh-r.top)/(vh+r.height*.5)).toFixed(4));
  }
  if(story&&innerWidth>=980&&!reduce.matches){
    const p=Number(story.style.getPropertyValue('--p'))*2.999,step=Math.floor(p),f=p-step;
    story.dataset.step=String(step);
    story.style.setProperty('--rot',-(Math.min(step,2)+(step<2?settle(f):0))*90+'deg');
    for(let i=0;i<3;i++)story.style.setProperty('--s'+i,String(i<step?1:i===step?clamp(f/.3):0));
  }
}
let queued=false;
const schedule=()=>{if(!queued){queued=true;requestAnimationFrame(()=>{queued=false;frame();});}};
addEventListener('scroll',schedule,{passive:true});
addEventListener('resize',()=>{measure();schedule();});
addEventListener('load',measure);
measure();frame();

const reveals=[...document.querySelectorAll<HTMLElement>('[data-reveal]')];
if(reduce.matches||!('IntersectionObserver' in window))reveals.forEach(el=>el.classList.add('is-in'));
else{const seen=new IntersectionObserver(entries=>{for(const entry of entries)if(entry.isIntersecting){entry.target.classList.add('is-in');seen.unobserve(entry.target);}},{rootMargin:'0px 0px -8% 0px'});reveals.forEach(el=>seen.observe(el));}

for(const card of document.querySelectorAll<HTMLElement>('[data-tilt]')){
  card.addEventListener('pointermove',e=>{
    if(reduce.matches||e.pointerType!=='mouse')return;
    const r=card.getBoundingClientRect(),x=(e.clientX-r.left)/r.width,y=(e.clientY-r.top)/r.height;
    card.style.setProperty('--ry',((x-.5)*8).toFixed(2)+'deg');card.style.setProperty('--rx',((.5-y)*8).toFixed(2)+'deg');
    card.style.setProperty('--mx',e.clientX-r.left+'px');card.style.setProperty('--my',e.clientY-r.top+'px');
  });
  card.addEventListener('pointerleave',()=>{card.style.removeProperty('--rx');card.style.removeProperty('--ry');});
}

const counter=document.querySelector<HTMLElement>('[data-count]');
if(counter&&!reduce.matches&&'IntersectionObserver' in window){
  const target=Number(counter.dataset.count);
  const watch=new IntersectionObserver(([entry])=>{
    if(!entry?.isIntersecting)return;watch.disconnect();const start=performance.now();
    const tick=(now:number)=>{const t=clamp((now-start)/1400);counter.textContent=String(Math.round(target*(1-Math.pow(1-t,4))));if(t<1)requestAnimationFrame(tick);};
    requestAnimationFrame(tick);
  });
  watch.observe(counter);
}
