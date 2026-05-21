import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Modal, Button, message, Spin, Progress, Tag, Typography, List, Statistic, Row, Col } from 'antd';
import { SafetyOutlined, CheckCircleOutlined, WarningOutlined, CloseCircleOutlined, ScanOutlined } from '@ant-design/icons';
import { batchCheckModSecurity, type BatchSecurityReport } from '../utils/advanced-features-api';

const { Text } = Typography;

import type { ModInfo } from '../utils/tauri-api';

interface SecurityScannerProps {
  visible: boolean;
  onClose: () => void;
  mods: ModInfo[];
}

function getRiskColor(risk: string) {
  switch (risk) {
    case 'Critical Risk': return '#ff4d4f';
    case 'High Risk': return '#ff7875';
    case 'Medium Risk': return '#faad14';
    default: return '#52c41a';
  }
}

function getRiskLabel(risk: string, t: (key: string) => string): string {
  const map: Record<string, string> = {
    'Critical Risk': t('app.features.security.criticalRisk'),
    'High Risk': t('app.features.security.highRisk'),
    'Medium Risk': t('app.features.security.mediumRisk'),
    'Low Risk': t('app.features.security.lowRisk'),
  };
  return map[risk] || risk;
}

function getCheckNameLabel(name: string, t: (key: string) => string): string {
  const map: Record<string, string> = {
    'DLL Detection': t('app.features.security.checkDll'),
    'Entry Point Verification': t('app.features.security.checkEntryPoint'),
    'Nexus Authentication': t('app.features.security.checkNexusAuth'),
    'Manifest Check': t('app.features.security.checkManifest'),
    'File Integrity': t('app.features.security.checkFileIntegrity'),
  };
  return map[name] || name;
}

function getScoreColor(score: number) {
  if (score >= 80) return '#52c41a';
  if (score >= 60) return '#faad14';
  if (score >= 40) return '#fa8c16';
  return '#ff4d4f';
}

export default function SecurityScanner({ visible, onClose, mods }: SecurityScannerProps) {
  const { t } = useTranslation();
  const [scanning, setScanning] = useState(false);
  const [report, setReport] = useState<BatchSecurityReport | null>(null);

  const handleScan = useCallback(async () => {
    setScanning(true);
    try {
      const modsData = mods.map(m => ({ folder_path: m.folder_path }));
      const result = await batchCheckModSecurity(modsData);
      setReport(result);
      message.success(t('features.security.scanComplete'));
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setScanning(false);
    }
  }, [mods, t]);

  const handleClose = useCallback(() => {
    setReport(null);
    onClose();
  }, [onClose]);

  return (
    <Modal
      title={t('features.security.title')}
      open={visible}
      onCancel={handleClose}
      width={900}
      footer={
        <div style={{ display: 'flex', gap: 8, justifyContent: 'space-between' }}>
          <Button icon={<ScanOutlined />} onClick={handleScan} loading={scanning} disabled={scanning || mods.length === 0}>
            {report ? t('features.security.rescan') : t('features.security.scanAll')}
          </Button>
          <Button onClick={handleClose}>{t('app.common.close')}</Button>
        </div>
      }
    >
      <Spin spinning={scanning}>
        {!report && !scanning && (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <SafetyOutlined style={{ fontSize: 48, color: '#1677ff' }} />
            <p style={{ marginTop: 16, color: 'var(--svl-text-secondary)' }}>
              {t('features.security.description')}
            </p>
            <Button type="primary" icon={<ScanOutlined />} onClick={handleScan}>
              {t('features.security.scanAll')}
            </Button>
          </div>
        )}

        {report && (
          <div>
            <Row gutter={16} style={{ marginBottom: 24 }}>
              <Col span={6}>
                <Statistic
                  title={t('features.security.avgScore')}
                  value={Math.round(report.average_score)}
                  suffix="/ 100"
                  valueStyle={{ color: getScoreColor(report.average_score) }}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title={t('features.security.lowRisk')}
                  value={report.low_risk_count}
                  valueStyle={{ color: '#52c41a' }}
                  prefix={<CheckCircleOutlined />}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title={t('features.security.mediumRisk')}
                  value={report.medium_risk_count}
                  valueStyle={{ color: '#faad14' }}
                  prefix={<WarningOutlined />}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title={t('features.security.highRisk')}
                  value={report.high_risk_count}
                  valueStyle={{ color: '#ff4d4f' }}
                  prefix={<CloseCircleOutlined />}
                />
              </Col>
            </Row>

            <List
              dataSource={report.reports}
              size="small"
              renderItem={(item) => (
                <List.Item style={{ padding: '8px 0' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 16, width: '100%' }}>
                    <Progress
                      type="circle"
                      percent={Math.round(item.security_score)}
                      width={50}
                      strokeColor={getScoreColor(item.security_score)}
                      format={() => Math.round(item.security_score)}
                    />
                    <div style={{ flex: 1 }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <Text strong>{item.mod_name}</Text>
                        <Tag color={getRiskColor(item.risk_level)}>{getRiskLabel(item.risk_level, t)}</Tag>
                      </div>
                      <Text type="secondary" style={{ fontSize: 12 }}>{item.unique_id}</Text>
                    </div>
                    <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', maxWidth: 300 }}>
                      {item.checks.map((check, i) => (
                        <Tag
                          key={i}
                          color={check.passed ? 'green' : check.severity === 'Medium' ? 'orange' : 'red'}
                          style={{ fontSize: 11 }}
                        >
                          {check.passed ? '✓' : '✗'} {getCheckNameLabel(check.check_name, t)}
                        </Tag>
                      ))}
                    </div>
                  </div>
                </List.Item>
              )}
            />
          </div>
        )}
      </Spin>
    </Modal>
  );
}
