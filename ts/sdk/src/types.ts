/**
 * Type definitions for Mycelix Supply Chain
 */

export type EventType = 'PRODUCED' | 'TRANSFORMED' | 'SHIPPED' | 'RECEIVED' | 'CERTIFIED';

export interface Location {
  lat?: number;
  lon?: number;
  address?: string;
  country?: string;
}

export interface Facility {
  id: string;
  name: string;
  location?: Location;
}

export interface Shipment {
  shipmentId: string;
  carrier?: string;
  trackingNumber?: string;
  origin?: string;
  destination?: string;
}

export interface Certification {
  certType: string;
  certBody: string;
  certId: string;
  validFrom: string;
  validUntil: string;
}

export interface CredentialSubject {
  eventType: EventType;
  productId: string;
  batchId: string;
  prevBatchIds?: string[];
  quantity: number;
  unit: string;
  facility: Facility;
  timestamp: string;
  shipment?: Shipment;
  certification?: Certification;
  metadata?: Record<string, any>;
}

export interface SupplyEventVC {
  '@context': string[];
  type: string[];
  issuer: string;
  issuanceDate: string;
  expirationDate?: string;
  credentialSubject: CredentialSubject;
  proof?: any;
}

export interface Lineage {
  hash: string;
  previousClaims?: string[];
}

export interface Subject {
  batchId: string;
  productId: string;
}

export interface Assertion {
  eventType: EventType;
  quantity?: number;
  unit?: string;
  facilityId?: string;
}

export interface Evidence {
  vcJwt: string;
  additionalDocuments?: Array<{
    type: string;
    uri: string;
    hash?: string;
  }>;
}

export interface DkgClaim {
  id: string;
  type: string;
  issuer: string;
  subject: Subject;
  assertion: Assertion;
  evidence: Evidence;
  lineage: Lineage;
  timestamp: string;
  confidence?: number;
  metadata?: Record<string, any>;
}

export interface EventResponse {
  vc_jwt: string;
  claim_id: string;
  lineage_hash: string;
  previous_claims?: string[];
}

export interface ClaimResponse {
  claim: DkgClaim;
  lineage?: DkgClaim[];
}

export interface VerifyRequest {
  vc_jwt: string;
  expected_product_id?: string;
  check_lineage?: boolean;
}

export interface VerifyResponse {
  valid: boolean;
  signature_valid: boolean;
  lineage_valid?: boolean;
  issuer?: string;
  errors?: string[];
}

export interface HealthResponse {
  status: string;
  version: string;
}
