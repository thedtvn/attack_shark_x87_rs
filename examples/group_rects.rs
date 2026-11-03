//! Print every KeyGroup rectangle (offline, no hardware).
use attack_shark_x87::KeyGroup;

fn main() {
    for g in KeyGroup::AREAS.into_iter().chain(KeyGroup::CLUSTERS) {
        let r = g.rect();
        println!("{:<20} x {:>3}..{:<3} y {:>3}..{:<3} ({}x{})", format!("{g:?}"), r.x1, r.x2, r.y1, r.y2, r.width(), r.height());
    }
    let a = KeyGroup::AREAS;
    for i in 0..a.len() {
        for j in i + 1..a.len() {
            let (p, q) = (a[i].rect(), a[j].rect());
            if p.x1 < q.x2 && q.x1 < p.x2 && p.y1 < q.y2 && q.y1 < p.y2 {
                println!("overlap: {:?} and {:?}", a[i], a[j]);
            }
        }
    }
}
