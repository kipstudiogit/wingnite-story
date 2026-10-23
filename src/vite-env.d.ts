/// <reference types="vite/client" />

declare module "*.css" {
  const styles: Record<string, string>;
  export default styles;
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

interface GPUAdapter {
  requestDevice(): Promise<GPUDevice>;
}

interface GPUQueue {
  writeBuffer(buffer: GPUBuffer, bufferOffset: number, data: BufferSource): void;
  submit(commandBuffers: GPUCommandBuffer[]): void;
}

interface GPUTextureView {}

interface GPUTexture {
  createView(): GPUTextureView;
}

interface GPUCanvasContext {
  configure(configuration: {
    device: GPUDevice;
    format: string;
    alphaMode?: "opaque" | "premultiplied";
  }): void;
  getCurrentTexture(): GPUTexture;
}

interface GPUShaderModule {}

interface GPURenderPipeline {}

interface GPUBuffer {
  readonly size: number;
  destroy(): void;
}

interface GPUCommandBuffer {}

interface GPURenderPassColorAttachment {
  view: GPUTextureView;
  clearValue?: { r: number; g: number; b: number; a: number };
  loadOp: "clear" | "load";
  storeOp: "store" | "discard";
}

interface GPURenderPassDescriptor {
  colorAttachments: GPURenderPassColorAttachment[];
}

interface GPURenderPassEncoder {
  setPipeline(pipeline: GPURenderPipeline): void;
  setVertexBuffer(slot: number, buffer: GPUBuffer | null): void;
  draw(vertexCount: number, instanceCount?: number, firstVertex?: number, firstInstance?: number): void;
  end(): void;
}

interface GPUCommandEncoder {
  beginRenderPass(descriptor: GPURenderPassDescriptor): GPURenderPassEncoder;
  finish(): GPUCommandBuffer;
}

interface GPUDevice {
  readonly queue: GPUQueue;
  createShaderModule(descriptor: { code: string }): GPUShaderModule;
  createRenderPipeline(descriptor: Record<string, unknown>): GPURenderPipeline;
  createBuffer(descriptor: { size: number; usage: number }): GPUBuffer;
  createCommandEncoder(): GPUCommandEncoder;
}

interface GPU {
  requestAdapter(): Promise<GPUAdapter | null>;
  getPreferredCanvasFormat(): string;
}

interface Navigator {
  readonly gpu?: GPU;
}