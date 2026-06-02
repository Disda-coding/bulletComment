export interface DanmakuMessage {
  type: "danmaku" | "system" | "online_count";
  content: string;
  color?: string;
  size?: number;
  action?: string;
  count?: number;
}

export interface ServerStatus {
  running: boolean;
  address: string;
  port: number;
  online_count: number;
}
