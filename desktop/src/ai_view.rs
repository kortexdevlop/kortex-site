use crate::{account, ai_bridge, config::Config};
use dioxus::prelude::*;
use serde_json::{Value, json};
const MAX_FILE: u64 = 700_000;
const IDEAS: [&str; 3] = [
    "Multiplie 25 par 48 puis ajoute 10",
    "Quels outils pour calculer le hash d’un fichier ?",
    "À quoi tu sers et que peux-tu faire ?",
];
#[derive(Clone, PartialEq)]
struct Run {
    steps: Vec<Value>,
    states: Vec<String>,
    results: Vec<String>,
    saves: Vec<Vec<(String, String)>>,
    files: Vec<Vec<u8>>,
    owner: Option<String>,
    created: u64,
    local: bool,
}
#[derive(Clone, PartialEq)]
struct Turn {
    user: bool,
    text: String,
    files: Vec<String>,
    run: Option<Run>,
}
fn attachment_kind(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if [".png", ".jpg", ".jpeg", ".webp"]
        .iter()
        .any(|e| lower.ends_with(e))
    {
        "image"
    } else if lower.ends_with(".pdf") {
        "pdf"
    } else if [".csv", ".txt", ".json", ".md"]
        .iter()
        .any(|e| lower.ends_with(e))
    {
        "text"
    } else {
        "other"
    }
}
fn clock(at_ms: u64) -> String {
    let local = kortex_runtime_native::permissions::now_ms();
    let minutes = at_ms.saturating_sub(local).div_ceil(60000);
    if minutes >= 60 {
        format!("dans {} h {:02}", minutes / 60, minutes % 60)
    } else {
        format!("dans {minutes} min")
    }
}
fn new_run(steps: Vec<Value>, files: Vec<Vec<u8>>, owner: Option<String>, local: bool) -> Run {
    let n = steps.len();
    Run {
        steps,
        states: vec!["À lancer".into(); n],
        results: vec![String::new(); n],
        saves: vec![Vec::new(); n],
        files,
        owner,
        created: kortex_runtime_native::permissions::now_ms(),
        local,
    }
}
fn file_name(tool: &str, data: &Value, key: &str) -> String {
    let format = data["format"]
        .as_str()
        .unwrap_or(if key == "pdf_base64" { "pdf" } else { "bin" });
    format!("{tool}.{}", if format == "jpeg" { "jpg" } else { format })
}
fn save(name: String, encoded: String, mut notice: Signal<String>) {
    spawn(async move {
        let written = tokio::task::spawn_blocking(move || {
            use base64::Engine;
            let Some(path) = rfd::FileDialog::new().set_file_name(&name).save_file() else {
                return Ok(false);
            };
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .map_err(|_| ())?;
            std::fs::write(path, bytes).map(|_| true).map_err(|_| ())
        })
        .await;
        match written {
            Ok(Ok(true)) => notice.set("Fichier enregistré.".into()),
            Ok(Ok(false)) => {}
            _ => notice.set("Enregistrement impossible.".into()),
        }
    });
}
#[component]
pub fn AiView(config: Config, catalogue: Vec<String>) -> Element {
    let mut message = use_signal(String::new);
    let mut consent = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut turns = use_signal(Vec::<Turn>::new);
    let mut notice = use_signal(String::new);
    let mut attached = use_signal(Vec::<(String, Vec<u8>)>::new);
    let mut quota = use_signal(|| Value::Null);
    let boot = config.clone();
    use_future(move || {
        let c = boot.clone();
        async move {
            if c.offline {
                return;
            }
            if let Ok(Ok(value)) = tokio::task::spawn_blocking(move || account::ai_quota(&c)).await
            {
                quota.set(value)
            }
        }
    });
    let send_config = config.clone();
    let run_config = config.clone();
    let offline = config.offline;
    let title = turns
        .read()
        .iter()
        .find(|t| t.user)
        .map(|t| t.text.clone())
        .unwrap_or_else(|| "Nouvelle conversation".into());
    let send = use_callback(move |_: ()| {
        let text = message.read().trim().to_owned();
        if *busy.peek() || !*consent.peek() || offline || text.is_empty() {
            return;
        }
        let c = send_config.clone();
        let catalogue = catalogue.clone();
        let files = std::mem::take(&mut *attached.write());
        let mut history: Vec<Value> = turns
            .peek()
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|t| {
                json!({"role":if t.user{"user"}else{"assistant"},"content":t.text.chars().take(800).collect::<String>()})
            })
            .collect();
        history.push(json!({"role":"user","content":text}));
        let kinds: Vec<&'static str> = files.iter().map(|(n, _)| attachment_kind(n)).collect();
        let owner = crate::secure_store::load(&c)
            .ok()
            .flatten()
            .map(|s| s.user_id.clone());
        turns.write().push(Turn {
            user: true,
            text: text.clone(),
            files: files.iter().map(|(n, _)| n.clone()).collect(),
            run: None,
        });
        message.set(String::new());
        notice.set(String::new());
        busy.set(true);
        spawn(async move {
            let answer = tokio::task::spawn_blocking(move || {
                account::ai_chat(&c, history, &catalogue, kinds)
            })
            .await;
            match answer {
                Ok(Ok(value)) if value["v"] == 2 && value["reply"].is_string() => {
                    let steps = value["steps"].as_array().cloned().unwrap_or_default();
                    if !value["quota"].is_null() {
                        quota.set(value["quota"].clone())
                    }
                    turns.write().push(Turn {
                        user: false,
                        text: value["reply"].as_str().unwrap_or_default().to_owned(),
                        files: Vec::new(),
                        run: (!steps.is_empty()).then(|| {
                            new_run(
                                steps,
                                files.into_iter().map(|(_, b)| b).collect(),
                                owner,
                                false,
                            )
                        }),
                    });
                }
                failure => {
                    turns.write().pop();
                    message.set(text);
                    attached.set(files);
                    notice.set(match failure {
                        Ok(Err(error)) => error,
                        _ => "Réponse invalide. Rien n’a été exécuté.".into(),
                    });
                }
            }
            busy.set(false);
        });
    });
    let launch = use_callback(move |index: usize| {
        if *busy.peek() {
            return;
        }
        let Some(run) = turns.peek().get(index).and_then(|t| t.run.clone()) else {
            return;
        };
        let c = run_config.clone();
        busy.set(true);
        spawn(async move {
            let mut outputs: Vec<Value> = Vec::new();
            for (position, step) in run.steps.iter().enumerate() {
                let set = |value: String, mut turns: Signal<Vec<Turn>>| {
                    if let Some(r) = turns.write()[index].run.as_mut() {
                        r.states[position] = value
                    }
                };
                set("Vérification locale…".into(), turns);
                let plan = ai_bridge::resolve_step(step, &run.files, &outputs)
                    .map(|s| json!({"v":1,"summary":"Action approuvée localement","steps":[s]}));
                let (cfg, owner, created) = (c.clone(), run.owner.clone(), run.created);
                let result = match plan {
                    Ok(p) => tokio::task::spawn_blocking(move || {
                        ai_bridge::execute_reviewed(&cfg, p, owner.as_deref(), created)
                    })
                    .await
                    .unwrap_or_else(|_| Err("Exécution interrompue.".into())),
                    Err(e) => Err(e),
                };
                match result {
                    Ok(value) => {
                        let shown = kortex_core::ai_bridge::local_result(value.clone())
                            .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                            .unwrap_or_else(|_| {
                                "Résultat trop volumineux pour être affiché.".into()
                            });
                        let tool = step["tool_id"].as_str().unwrap_or("resultat");
                        let saves = kortex_core::ai_bridge::carried(&value)
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    file_name(tool, &value, &k),
                                    v.as_str().unwrap_or("").to_owned(),
                                )
                            })
                            .collect();
                        if let Some(r) = turns.write()[index].run.as_mut() {
                            r.states[position] = "Exécutée localement".into();
                            r.results[position] = shown;
                            r.saves[position] = saves;
                        }
                        outputs.push(value);
                    }
                    Err(error) => {
                        if let Some(r) = turns.write()[index].run.as_mut() {
                            r.states[position] = error;
                            for later in r.states.iter_mut().skip(position + 1) {
                                *later = "Non lancée".into()
                            }
                        }
                        break;
                    }
                }
            }
            busy.set(false);
        });
    });
    let attach = move |_| {
        spawn(async move {
            let picked = tokio::task::spawn_blocking(|| {
                rfd::FileDialog::new()
                    .pick_files()
                    .unwrap_or_default()
                    .into_iter()
                    .take(2)
                    .map(|path| {
                        let size = std::fs::metadata(&path).map_or(u64::MAX, |m| m.len());
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "fichier".into());
                        (name, size, std::fs::read(path).unwrap_or_default())
                    })
                    .collect::<Vec<_>>()
            })
            .await
            .unwrap_or_default();
            if picked.iter().any(|(_, size, _)| *size > MAX_FILE) {
                notice.set("L’assistant accepte des fichiers de 700 Ko maximum. Ouvrez l’outil pour les plus gros.".into())
            }
            let mut list = attached.write();
            list.extend(
                picked
                    .into_iter()
                    .filter(|(_, size, _)| *size <= MAX_FILE)
                    .map(|(n, _, b)| (n, b)),
            );
            list.truncate(2);
        });
    };
    let list = turns.read().clone();
    let credits = format!(
        "{} / {} crédits · recharge {}",
        quota.read()["remaining"],
        quota.read()["limit"],
        clock(quota.read()["resets_at"].as_u64().unwrap_or(0))
    );
    rsx! {
    div{class:"ai-view",
    div{class:"ai-subbar",p{"{title}"}
    if quota.read()["limit"].is_u64(){div{class:"ai-quota",span{"{credits}"}meter{min:0,max:quota.read()["limit"].as_u64().unwrap_or(1) as f64,value:quota.read()["remaining"].as_u64().unwrap_or(0) as f64,aria_label:"Crédits IA restants"}}}
    button{disabled:*busy.read(),onclick:move |_|{let owner=None;let step=json!({"tool_id":"calculatrice","action_id":"multiply","input":{"a":25,"b":48},"bind":{},"requires_consent":true});let mut t=turns.write();t.push(Turn{user:true,text:"Exemple local : multiplier 25 par 48".into(),files:Vec::new(),run:None});t.push(Turn{user:false,text:"Exemple local, sans IA. La calculatrice doit être installée.".into(),files:Vec::new(),run:Some(new_run(vec![step],Vec::new(),owner,true))});},"Essayer un exemple local"}
    button{disabled:*busy.read(),onclick:move |_|{turns.write().clear();notice.set(String::new());},"Nouvelle conversation"}
    }
    div{class:"ai-scroll",
    if list.is_empty(){div{class:"ai-empty",span{class:"ai-mark","✦"}h2{"Bonjour, comment puis-je vous aider ?"}p{"Discutez avec l’assistant ou demandez-lui d’enchaîner plusieurs outils. Vos fichiers restent sur cet appareil."}
    div{class:"ai-ideas",for idea in IDEAS{button{onclick:move |_|message.set(idea.into()),"{idea}"}}}}}
    else{div{class:"ai-thread",
    for (index,turn) in list.into_iter().enumerate(){
    if turn.user{div{class:"ai-user","{turn.text}",if !turn.files.is_empty(){small{{turn.files.join(" · ")}}}}}
    else{section{class:"ai-reply",aria_label:"Réponse de l’assistant",span{class:"ai-mark","✦"}div{class:"ai-reply-body",p{class:"ai-text","{turn.text}"}
    if let Some(run)=turn.run{div{class:"ai-chain",
    h2{if run.local{"Action locale"}else if run.steps.len()>1{"{run.steps.len()} actions à la suite"}else{"Action proposée"}}
    for (position,step) in run.steps.iter().cloned().enumerate(){
    article{class:"ai-step",h3{{format!("{}. {} · {}",position+1,step["tool_id"].as_str().unwrap_or(""),step["action_id"].as_str().unwrap_or(""))}}
    pre{{serde_json::to_string_pretty(&step["input"]).unwrap_or_default()}{step["bind"].as_object().into_iter().flatten().map(|(k,b)|format!("\n{k} ← {}",if let Some(f)=b["file"].as_u64(){format!("votre fichier {}",f+1)}else{format!("étape {} · {}",b["step"].as_u64().unwrap_or(0)+1,b["output"].as_str().unwrap_or(""))})).collect::<String>()}}
    p{"{run.states[position]}"}
    if !run.results[position].is_empty(){div{class:"ai-result",strong{"Résultat local — aucun envoi au cloud"}pre{"{run.results[position]}"}
    for (name,encoded) in run.saves[position].clone(){{let label=name.clone();rsx!{button{class:"ai-save",onclick:move |_|save(name.clone(),encoded.clone(),notice),"Enregistrer {label}"}}}}
    }}}}
    if run.states.iter().all(|s|s=="À lancer"){div{class:"ai-decide",
    button{class:"ai-go",disabled:*busy.read(),onclick:move |_|launch.call(index),if run.steps.len()>1{"Lancer les {run.steps.len()} actions"}else{"Lancer l’action"}}
    button{disabled:*busy.read(),onclick:move |_|{if let Some(r)=turns.write()[index].run.as_mut(){r.states.iter_mut().for_each(|s|*s="Refusée".into())}},"Refuser"}}}
    }}}}}}
    if *busy.read(){p{class:"ai-typing",role:"status","Kortex IA travaille…"}}
    }}}
    form{class:"ai-composer",onsubmit:move |e|{e.prevent_default();send.call(())},
    p{class:"ai-notice",role:"status","{notice}"}
    if offline{p{class:"ai-notice","IA indisponible hors connexion. Vos outils installés restent disponibles."}}
    if !attached.read().is_empty(){ul{class:"ai-files",aria_label:"Fichiers joints",for (i,(name,_)) in attached.read().iter().enumerate(){li{"{name}",button{r#type:"button",aria_label:"Retirer {name}",onclick:move |_|{attached.write().remove(i);},"×"}}}}}
    div{class:"ai-input",
    button{r#type:"button",class:"ai-attach",aria_label:"Joindre un fichier",title:"Joindre un fichier (il reste sur cet appareil)",disabled:*busy.read()||attached.read().len()>=2,onclick:attach,"+"}
    textarea{aria_label:"Votre message",rows:1,maxlength:1000,placeholder:"Écrivez à Kortex IA…",value:"{message}",oninput:move|e|message.set(e.value()),onkeydown:move|e|{if e.key()==Key::Enter&&!e.modifiers().shift(){e.prevent_default();send.call(())}}}
    button{r#type:"submit",aria_label:"Envoyer",disabled:*busy.read()||!*consent.read()||offline||message.read().trim().is_empty(),"↑"}
    }
    label{class:"ai-consent",input{r#type:"checkbox",checked:*consent.read(),onchange:move|e|consent.set(e.checked())}" J’autorise l’envoi de mes messages, sans données privées, à Workers AI. Mes fichiers ne sont jamais envoyés."}
    }
    }
    }
}
