use crate::{PdfTool, explain};
use serde_json::{Value, json};
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.into()
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}
fn kilobytes(bytes: usize) -> String {
    format!("{} Ko", bytes.div_ceil(1000))
}
fn button(key: &str, text: &str, disabled: bool) -> Value {
    node(
        "button",
        key,
        json!({"text":text,"disabled":disabled}),
        vec![],
    )
}
fn slot_card(
    tool: &PdfTool,
    index: usize,
    title: &str,
    pick: &str,
    target: &str,
    disabled: bool,
) -> Value {
    let name = |text: &str| {
        node(
            "text",
            &format!("name_{index}"),
            json!({"text":text}),
            vec![],
        )
    };
    let children = match &tool.slots[index] {
        Some(slot) => {
            let details = format!("{} page(s) · {}", slot.pages, kilobytes(slot.data.len()));
            vec![
                name(&clip(&slot.name, 100)),
                node(
                    "text",
                    &format!("details_{index}"),
                    json!({"text":details}),
                    vec![],
                ),
                button(pick, &format!("Remplacer {target}"), disabled),
            ]
        }
        None => vec![
            name("Aucun fichier"),
            button(pick, &format!("Choisir {target}"), disabled),
        ],
    };
    node(
        "card",
        &format!("slot_{index}"),
        json!({"title":title}),
        children,
    )
}
impl PdfTool {
    pub(crate) fn render(&self) -> Value {
        let busy = self.host.pending_count() > 0;
        let loading = ["picking", "reading"].contains(&self.phase);
        let ready = self.slots.iter().all(Option::is_some);
        let message = match (self.phase, &self.merged) {
            ("picking", _) => "Choisissez un fichier PDF après autorisation.".to_string(),
            ("reading", _) => format!("Lecture locale : {} / {} octets", self.offset, self.size),
            ("merged", Some((bytes, pages))) => format!(
                "Fusion terminée : {pages} pages, {}. Aucun fichier envoyé.",
                kilobytes(bytes.len())
            ),
            ("cancelled", _) => {
                "Opération annulée. Les fichiers déjà chargés sont conservés.".into()
            }
            ("error", _) => format!("{} ({})", explain(&self.error), self.error),
            _ if ready => "Les deux PDF sont prêts à être fusionnés.".into(),
            _ => "Choisissez deux PDF non protégés de 700 Ko maximum chacun.".into(),
        };
        let mut actions = Vec::new();
        if loading {
            actions.push(button("cancel", "Annuler", false));
        } else if self.merged.is_some() {
            actions.push(button("save", "Enregistrer le PDF fusionné", busy));
        } else if ready {
            actions.push(button("merge", "Fusionner", busy));
        }
        if ready && !loading {
            actions.push(button("swap", "Inverser l’ordre", busy));
        }
        if self.slots.iter().any(Option::is_some) {
            actions.push(button("reset", "Recommencer", busy && !loading));
        }
        let mut children = vec![
            node(
                "heading",
                "title",
                json!({"text":"Fusion PDF","level":1}),
                vec![],
            ),
            node(
                "text",
                "intro",
                json!({"text":"Les pages du premier PDF sont suivies de celles du second. Signets, formulaires et destinations nommées ne sont pas conservés."}),
                vec![],
            ),
            node(
                "grid",
                "files",
                json!({"columns":2}),
                vec![
                    slot_card(
                        self,
                        0,
                        "Premier PDF",
                        "pick_first",
                        "le premier PDF",
                        busy || loading,
                    ),
                    slot_card(
                        self,
                        1,
                        "Second PDF",
                        "pick_second",
                        "le second PDF",
                        busy || loading,
                    ),
                ],
            ),
        ];
        if self.phase == "reading" {
            children.push(node(
                "progress",
                "progress",
                json!({"label":"Progression","value":self.offset as f64 / self.size as f64 * 100.0}),
                vec![],
            ));
        }
        children.push(node(
            "alert",
            "status",
            json!({"text":message,"severity":if self.phase=="error"{"error"}else{"info"}}),
            vec![],
        ));
        if !actions.is_empty() {
            children.push(node("row", "actions", json!({"gap":8}), actions));
        }
        if !self.notice.is_empty() {
            children.push(node(
                "alert",
                "notice",
                json!({"text":clip(&self.notice,500),"severity":"info"}),
                vec![],
            ));
        }
        node("column", "pdf-tool", json!({"gap":12}), children)
    }
}
