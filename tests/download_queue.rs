use speedflow::core::download_queue::{DownloadQueue, Priority};
use uuid::Uuid;

#[test]
fn test_queue_priority_ordering() {
    let mut queue = DownloadQueue::new(3);

    let id_low = Uuid::new_v4();
    let id_normal = Uuid::new_v4();
    let id_urgent = Uuid::new_v4();

    queue.push(id_low, Priority::Low);
    queue.push(id_urgent, Priority::Urgent);
    queue.push(id_normal, Priority::Normal);

    // Urgent should come first
    assert_eq!(queue.pop_next(), Some(id_urgent));
    // Then Normal
    assert_eq!(queue.pop_next(), Some(id_normal));
    // Then Low
    assert_eq!(queue.pop_next(), Some(id_low));
    assert_eq!(queue.pop_next(), None);
}
