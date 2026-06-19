import { Skeleton, Space } from 'antd';

export default function ToolSkeleton() {
  return (
    <div style={{ padding: '24px 28px', maxWidth: 1100, margin: '0 auto' }}>
      <Skeleton active paragraph={{ rows: 1 }} style={{ marginBottom: 24, maxWidth: 320 }} />
      <Skeleton.Node active style={{ width: 220, height: 80, marginBottom: 24, borderRadius: 10 }}>
        <span />
      </Skeleton.Node>
      <Space direction="vertical" size={16} style={{ width: '100%' }}>
        <Skeleton active paragraph={{ rows: 4 }} />
        <Skeleton active paragraph={{ rows: 3 }} />
        <Skeleton active paragraph={{ rows: 2 }} />
      </Space>
    </div>
  );
}
