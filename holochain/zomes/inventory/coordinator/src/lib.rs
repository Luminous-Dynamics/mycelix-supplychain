//! Inventory Coordinator Zome - Business logic for inventory management
use hdk::prelude::*;
use inventory_integrity::*;

fn ensure_path(path: Path, link_type: LinkTypes) -> ExternResult<EntryHash> {
    let typed = path.typed(link_type)?;
    typed.ensure()?;
    typed.path_entry_hash()
}

#[hdk_extern]
pub fn create_item(item: InventoryItem) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::InventoryItem(item.clone()))?;
    let all_path = Path::from("all_items");
    let all_hash = ensure_path(all_path, LinkTypes::AllItems)?;
    create_link(all_hash, action_hash.clone(), LinkTypes::AllItems, ())?;
    let cat_path = Path::from(format!("category/{}", item.category.to_lowercase()));
    let cat_hash = ensure_path(cat_path, LinkTypes::CategoryToItems)?;
    create_link(cat_hash, action_hash.clone(), LinkTypes::CategoryToItems, ())?;
    Ok(action_hash)
}

#[hdk_extern]
pub fn get_item(hash: ActionHash) -> ExternResult<Option<InventoryItem>> {
    match get(hash, GetOptions::default())? {
        Some(r) => Ok(r.entry().to_app_option().map_err(|e| wasm_error!(e))?),
        None => Ok(None),
    }
}

#[hdk_extern]
pub fn get_all_items(_: ()) -> ExternResult<Vec<InventoryItem>> {
    let path = Path::from("all_items");
    let typed = path.typed(LinkTypes::AllItems)?;
    let filter = LinkTypeFilter::try_from(LinkTypes::AllItems)?;
    let links = get_links(LinkQuery::new(typed.path_entry_hash()?, filter), GetStrategy::default())?;
    let mut items = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(item) = get_item(hash)? { items.push(item); }
        }
    }
    Ok(items)
}

#[hdk_extern]
pub fn update_stock(input: StockLevel) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::StockLevel(input.clone()))?;
    create_link(input.item_hash, action_hash.clone(), LinkTypes::ItemToStockLevels, ())?;
    let loc_path = Path::from(format!("location/{}", input.location));
    let loc_hash = ensure_path(loc_path, LinkTypes::LocationToStock)?;
    create_link(loc_hash, action_hash.clone(), LinkTypes::LocationToStock, ())?;
    Ok(action_hash)
}

#[hdk_extern]
pub fn record_movement(input: StockMovement) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::StockMovement(input.clone()))?;
    create_link(input.item_hash, action_hash.clone(), LinkTypes::ItemToMovements, ())?;
    Ok(action_hash)
}

#[hdk_extern]
pub fn get_stock_levels(item_hash: ActionHash) -> ExternResult<Vec<StockLevel>> {
    let filter = LinkTypeFilter::try_from(LinkTypes::ItemToStockLevels)?;
    let links = get_links(LinkQuery::new(item_hash, filter), GetStrategy::default())?;
    let mut levels = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(record) = get(hash, GetOptions::default())? {
                if let Some(level) = record.entry().to_app_option::<StockLevel>().map_err(|e| wasm_error!(e))? {
                    levels.push(level);
                }
            }
        }
    }
    Ok(levels)
}
