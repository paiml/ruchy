#[derive(Debug, Clone, PartialEq)]
enum RhlAction {
    FileTicket { repo: String, title: String, label: String },
}
#[derive(Clone)]
struct Facts {
    disk_free_of_root: i64,
}
fn decide(facts: Facts) -> Vec<RhlAction> {
    let mut plan: Vec<RhlAction> = Vec::new();
    {
        let free = facts.disk_free_of_root;
        {
            if free < 100 * 1000000000 {
                plan.push(RhlAction::FileTicket {
                    repo: "paiml/infra".to_string(),
                    title: "gx10 disk below 100 GB".to_string(),
                    label: "fleet".to_string(),
                })
            }
            plan
        }
    }
}
fn main() {
    {
        let low = decide(Facts {
            disk_free_of_root: 90 * 1000000000,
        });
        {
            let ok = decide(Facts {
                disk_free_of_root: 400 * 1000000000,
            });
            println!("{} {}", low.len(), ok.len())
        }
    }
}
