//! Payments Coordinator Zome - Business logic for payment processing
use hdk::prelude::*;
use payments_integrity::*;

fn ensure_path(path: Path, link_type: LinkTypes) -> ExternResult<EntryHash> {
    let typed = path.typed(link_type)?;
    typed.ensure()?;
    typed.path_entry_hash()
}

#[hdk_extern]
pub fn create_payment(payment: Payment) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::Payment(payment.clone()))?;
    create_link(payment.po_hash, action_hash.clone(), LinkTypes::PoToPayments, ())?;

    let payer_path = Path::from(format!("payer/{}", payment.payer));
    let payer_hash = ensure_path(payer_path, LinkTypes::PayerToPayments)?;
    create_link(payer_hash, action_hash.clone(), LinkTypes::PayerToPayments, ())?;

    let payee_path = Path::from(format!("payee/{}", payment.payee));
    let payee_hash = ensure_path(payee_path, LinkTypes::PayeeToPayments)?;
    create_link(payee_hash, action_hash.clone(), LinkTypes::PayeeToPayments, ())?;

    Ok(action_hash)
}

#[hdk_extern]
pub fn get_payment(hash: ActionHash) -> ExternResult<Option<Payment>> {
    match get(hash, GetOptions::default())? {
        Some(r) => Ok(r.entry().to_app_option().map_err(|e| wasm_error!(e))?),
        None => Ok(None),
    }
}

#[hdk_extern]
pub fn update_payment_status(input: (ActionHash, PaymentStatus)) -> ExternResult<ActionHash> {
    let (hash, new_status) = input;
    if let Some(record) = get(hash.clone(), GetOptions::default())? {
        if let Some(mut payment) = record.entry().to_app_option::<Payment>().map_err(|e| wasm_error!(e))? {
            payment.status = new_status.clone();
            if new_status == PaymentStatus::Completed {
                payment.completed_at = Some(sys_time()?);
            }
            return update_entry(hash, EntryTypes::Payment(payment));
        }
    }
    Err(wasm_error!(WasmErrorInner::Guest("Payment not found".into())))
}

#[hdk_extern]
pub fn create_invoice(invoice: Invoice) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::Invoice(invoice.clone()))?;
    create_link(invoice.po_hash, action_hash.clone(), LinkTypes::PoToInvoices, ())?;
    Ok(action_hash)
}

#[hdk_extern]
pub fn get_invoice(hash: ActionHash) -> ExternResult<Option<Invoice>> {
    match get(hash, GetOptions::default())? {
        Some(r) => Ok(r.entry().to_app_option().map_err(|e| wasm_error!(e))?),
        None => Ok(None),
    }
}

#[hdk_extern]
pub fn create_escrow(escrow: EscrowAccount) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::EscrowAccount(escrow.clone()))?;
    create_link(escrow.po_hash, action_hash.clone(), LinkTypes::PoToEscrow, ())?;
    Ok(action_hash)
}

#[hdk_extern]
pub fn fund_escrow(hash: ActionHash) -> ExternResult<ActionHash> {
    if let Some(record) = get(hash.clone(), GetOptions::default())? {
        if let Some(mut escrow) = record.entry().to_app_option::<EscrowAccount>().map_err(|e| wasm_error!(e))? {
            escrow.funded_at = Some(sys_time()?);
            return update_entry(hash, EntryTypes::EscrowAccount(escrow));
        }
    }
    Err(wasm_error!(WasmErrorInner::Guest("Escrow not found".into())))
}

#[hdk_extern]
pub fn release_escrow(hash: ActionHash) -> ExternResult<ActionHash> {
    if let Some(record) = get(hash.clone(), GetOptions::default())? {
        if let Some(mut escrow) = record.entry().to_app_option::<EscrowAccount>().map_err(|e| wasm_error!(e))? {
            if escrow.funded_at.is_none() {
                return Err(wasm_error!(WasmErrorInner::Guest("Escrow not funded".into())));
            }
            escrow.released_at = Some(sys_time()?);
            return update_entry(hash, EntryTypes::EscrowAccount(escrow));
        }
    }
    Err(wasm_error!(WasmErrorInner::Guest("Escrow not found".into())))
}

#[hdk_extern]
pub fn get_po_payments(po_hash: ActionHash) -> ExternResult<Vec<Payment>> {
    let filter = LinkTypeFilter::try_from(LinkTypes::PoToPayments)?;
    let links = get_links(LinkQuery::new(po_hash, filter), GetStrategy::default())?;
    let mut payments = Vec::new();
    for link in links {
        if let Some(hash) = link.target.into_action_hash() {
            if let Some(payment) = get_payment(hash)? { payments.push(payment); }
        }
    }
    Ok(payments)
}
