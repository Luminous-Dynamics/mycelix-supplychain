/**
 * Mycelix Supply Chain API Client
 */

import axios, { AxiosInstance } from 'axios';
import {
  SupplyEventVC,
  EventResponse,
  ClaimResponse,
  VerifyRequest,
  VerifyResponse,
  HealthResponse,
} from './types';

export interface ClientConfig {
  baseUrl: string;
  timeout?: number;
  headers?: Record<string, string>;
}

export class SupplyChainClient {
  private client: AxiosInstance;

  constructor(config: ClientConfig) {
    this.client = axios.create({
      baseURL: config.baseUrl,
      timeout: config.timeout || 30000,
      headers: {
        'Content-Type': 'application/json',
        ...config.headers,
      },
    });
  }

  /**
   * Check service health
   */
  async health(): Promise<HealthResponse> {
    const response = await this.client.get<HealthResponse>('/health');
    return response.data;
  }

  /**
   * Ingest a supply chain event
   */
  async ingestEvent(event: SupplyEventVC): Promise<EventResponse> {
    const response = await this.client.post<EventResponse>('/v1/events', event);
    return response.data;
  }

  /**
   * Get a claim by ID
   */
  async getClaim(claimId: string, includeLineage = true): Promise<ClaimResponse> {
    const response = await this.client.get<ClaimResponse>(`/v1/claims/${claimId}`, {
      params: { include_lineage: includeLineage },
    });
    return response.data;
  }

  /**
   * Get lineage for a batch
   */
  async getBatchLineage(batchId: string): Promise<ClaimResponse> {
    const response = await this.client.get<ClaimResponse>(`/v1/batches/${batchId}/lineage`);
    return response.data;
  }

  /**
   * Verify a Verifiable Credential
   */
  async verify(request: VerifyRequest): Promise<VerifyResponse> {
    const response = await this.client.post<VerifyResponse>('/v1/verify', request);
    return response.data;
  }

  /**
   * Helper: Create a PRODUCED event
   */
  createProducedEvent(params: {
    issuer: string;
    productId: string;
    batchId: string;
    quantity: number;
    unit: string;
    facility: { id: string; name: string };
    timestamp?: string;
    metadata?: Record<string, any>;
  }): SupplyEventVC {
    return {
      '@context': ['https://www.w3.org/2018/credentials/v1', 'https://mycelix.org/contexts/supply-chain/v1'],
      type: ['VerifiableCredential', 'SupplyChainEvent'],
      issuer: params.issuer,
      issuanceDate: new Date().toISOString(),
      credentialSubject: {
        eventType: 'PRODUCED',
        productId: params.productId,
        batchId: params.batchId,
        quantity: params.quantity,
        unit: params.unit,
        facility: params.facility,
        timestamp: params.timestamp || new Date().toISOString(),
        metadata: params.metadata,
      },
    };
  }

  /**
   * Helper: Create a SHIPPED event
   */
  createShippedEvent(params: {
    issuer: string;
    productId: string;
    batchId: string;
    quantity: number;
    unit: string;
    facility: { id: string; name: string };
    shipment: {
      shipmentId: string;
      carrier?: string;
      trackingNumber?: string;
      origin?: string;
      destination?: string;
    };
    timestamp?: string;
  }): SupplyEventVC {
    return {
      '@context': ['https://www.w3.org/2018/credentials/v1', 'https://mycelix.org/contexts/supply-chain/v1'],
      type: ['VerifiableCredential', 'SupplyChainEvent'],
      issuer: params.issuer,
      issuanceDate: new Date().toISOString(),
      credentialSubject: {
        eventType: 'SHIPPED',
        productId: params.productId,
        batchId: params.batchId,
        quantity: params.quantity,
        unit: params.unit,
        facility: params.facility,
        timestamp: params.timestamp || new Date().toISOString(),
        shipment: params.shipment,
      },
    };
  }

  /**
   * Helper: Create a TRANSFORMED event
   */
  createTransformedEvent(params: {
    issuer: string;
    productId: string;
    batchId: string;
    prevBatchIds: string[];
    quantity: number;
    unit: string;
    facility: { id: string; name: string };
    timestamp?: string;
    metadata?: Record<string, any>;
  }): SupplyEventVC {
    return {
      '@context': ['https://www.w3.org/2018/credentials/v1', 'https://mycelix.org/contexts/supply-chain/v1'],
      type: ['VerifiableCredential', 'SupplyChainEvent'],
      issuer: params.issuer,
      issuanceDate: new Date().toISOString(),
      credentialSubject: {
        eventType: 'TRANSFORMED',
        productId: params.productId,
        batchId: params.batchId,
        prevBatchIds: params.prevBatchIds,
        quantity: params.quantity,
        unit: params.unit,
        facility: params.facility,
        timestamp: params.timestamp || new Date().toISOString(),
        metadata: params.metadata,
      },
    };
  }
}
