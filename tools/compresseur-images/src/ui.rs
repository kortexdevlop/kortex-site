use crate::{ImageTool, explain};
use serde_json::{Value, json};

fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}

fn kilobytes(size: usize) -> String {
    format!("{} Ko", size.div_ceil(1000))
}

impl ImageTool {
    pub(crate) fn render(&self) -> Value {
        let busy = self.busy();
        let loading = ["picking", "reading"].contains(&self.phase);
        let status = match self.phase {
            "picking" => "Choisissez une image après autorisation.".into(),
            "reading" => format!("Lecture locale : {} / {} octets", self.offset, self.size),
            "ready" => format!(
                "{} prête à compresser ({}).",
                self.name,
                kilobytes(self.buffer.len())
            ),
            "compressed" => {
                let (bytes, format, width, height, original, reduced) =
                    self.result.as_ref().unwrap();
                let result = if *reduced {
                    format!(
                        "Image compressée : {width} × {height}, {} → {} ({format}).",
                        kilobytes(*original),
                        kilobytes(bytes.len())
                    )
                } else {
                    format!(
                        "Aucune réduction obtenue : l’original est conservé ({}).",
                        kilobytes(*original)
                    )
                };
                result
            }
            "cancelled" => "Opération annulée.".into(),
            "error" => format!("{} ({})", explain(&self.error), self.error),
            _ => "Choisissez un JPEG, PNG ou WebP de 700 Ko maximum et 4 mégapixels.".into(),
        };
        let mut children = vec![
            node(
                "heading",
                "title",
                json!({"text":"Compresseur d’images","level":1}),
                vec![],
            ),
            node(
                "text",
                "intro",
                json!({"text":"Traitement sur cet appareil. La transparence est conservée ; les images opaques sont exportées en JPEG. Si la sortie est plus lourde, l’original est conservé."}),
                vec![],
            ),
            node(
                "row",
                "pickers",
                json!({"gap":8}),
                vec![
                    node(
                        "button",
                        "pick",
                        json!({"text":"Choisir une image","disabled":busy||loading}),
                        vec![],
                    ),
                    node(
                        "button",
                        "cancel",
                        json!({"text":"Annuler","disabled":!loading}),
                        vec![],
                    ),
                ],
            ),
            node(
                "select",
                "quality",
                json!({
                    "label":"Qualité JPEG",
                    "value":self.quality.to_string(),
                    "disabled":busy,
                    "options":[
                        {"label":"45 — fichier plus léger","value":"45"},
                        {"label":"60 — équilibrée","value":"60"},
                        {"label":"75 — bonne qualité","value":"75"},
                        {"label":"85 — qualité élevée","value":"85"}
                    ]
                }),
                vec![],
            ),
            node(
                "progress",
                "progress",
                json!({
                    "label":"Progression de la lecture",
                    "value":if self.phase=="ready"||self.phase=="compressed"{100.0}else if self.size==0{0.0}else{self.offset as f64/self.size as f64*100.0}
                }),
                vec![],
            ),
            node(
                "alert",
                "status",
                json!({"text":status,"severity":if self.phase=="error"{"error"}else{"info"}}),
                vec![],
            ),
            node(
                "row",
                "actions",
                json!({"gap":8}),
                vec![
                    node(
                        "button",
                        "compress",
                        json!({"text":"Compresser l’image","disabled":busy||self.phase!="ready"}),
                        vec![],
                    ),
                    node(
                        "button",
                        "save",
                        json!({"text":"Enregistrer l’image","disabled":busy||self.result.is_none()}),
                        vec![],
                    ),
                ],
            ),
        ];
        if !self.notice.is_empty() {
            children.push(node(
                "alert",
                "notice",
                json!({"text":self.notice,"severity":"info"}),
                vec![],
            ));
        }
        node("column", "image-compressor", json!({"gap":12}), children)
    }
}
