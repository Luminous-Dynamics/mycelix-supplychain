//! Logistics Coordinator Zome - Business logic for shipments
use hdk::prelude::*;
use logistics_integrity::*;

fn ensure_path(path: Path, link_type: LinkTypes) -> ExternResult<EntryHash> {
    let typed = path.typed(link_type)?;
    typed.ensure()?;
    typed.path_entry_hash()
}

#[hdk_extern]
pub fn create_shipment(shipment: Shipment) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::Shipment(shipment.clone()))?;

    let sender_path = Path::from(format!("sender/{}", shipment.sender));
    let sender_hash = ensure_path(sender_path, LinkTypes::SenderToShipments)?;
    create_link(sender_hash, action_hash.clone(), LinkTypes::SenderToShipments, ())?;

    let recipient_path = Path::from(format!("recipient/{}", shipment.recipient));
    let recipient_hash = ensure_path(recipient_path, LinkTypes::RecipientToShipments)?;
    create_link(recipient_hash, action_hash.clone(), LinkTypes::RecipientToShipments, ())?;

    if let Some(po_hash) = shipment.po_hash {
        create_link(po_hash, action_hash.clone(), LinkTypes::PoToShipments, ())?;
    }
    Ok(action_hash)
}

#[hdk_extern]
pub fn get_shipment(hash: ActionHash) -> ExternResult<Option<Shipment>> {
    match get(hash, GetOptions::default())? {
        Some(r) => Ok(r.entry().to_app_option().map_err(|e| wasm_error!(e))?),
        None => Ok(None),
    }
}

#[hdk_extern]
pub fn add_tracking_event(event: TrackingEvent) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::TrackingEvent(event.clone()))?;
    create_link(event.shipment_hash.clone(), action_hash.clone(), LinkTypes::ShipmentToEvents, ())?;

    // Update shipment status
    if let Some(record) = get(event.shipment_hash.clone(), GetOptions::default())? {
        if let Some(mut shipment) = record.entry().to_app_option::<Shipment>().map_err(|e| wasm_error!(e))? {
            shipment.status = event.status.clone();
            if event.status == ShipmentStatus::Delivered {
                shipment.actual_delivery = Some(event.occurred_at);
            }
            update_entry(event.shipment_hash, EntryTypes::Shipment(shipment))?;
        }
    }
    Ok(action_hash)
}

#[hdk_extern]
pub fn get_tracking_events(shipment_hash: ActionHash) -> ExternResult<Vec<TrackingEvent>> {
    let filter = LinkTypeFilter::try_from(LinkTypes::ShipmentToEvents)?;
    let links = get_links(LinkQuery::new(shipment_hash, filter), GetStrategy::default())?;
    let mut events = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                if let Some(event) = record.entry().to_app_option::<TrackingEvent>().map_err(|e| wasm_error!(e))? {
                    events.push(event);
                }
            }
        }
    }
    Ok(events)
}

#[hdk_extern]
pub fn get_my_shipments(_: ()) -> ExternResult<Vec<Shipment>> {
    let my_agent = agent_info()?.agent_initial_pubkey;
    let sender_path = Path::from(format!("sender/{}", my_agent));
    let typed = sender_path.typed(LinkTypes::SenderToShipments)?;
    let filter = LinkTypeFilter::try_from(LinkTypes::SenderToShipments)?;
    let links = get_links(LinkQuery::new(typed.path_entry_hash()?, filter), GetStrategy::default())?;
    let mut shipments = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(s) = get_shipment(hash)? { shipments.push(s); }
        }
    }
    Ok(shipments)
}
