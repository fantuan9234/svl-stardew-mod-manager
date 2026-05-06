import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Card, Button, Spin, Alert, Progress, Space } from 'antd';
import { CheckCircleOutlined, CloseCircleOutlined, ReloadOutlined, GlobalOutlined } from '@ant-design/icons';
import { diagnoseNetwork, type NetworkDiagnosticResult } from '../utils/tauri-api';

export default function NetworkDiagnostic() {
  const { t } = useTranslation();
  const [diagnosing, setDiagnosing] = useState(false);
  const [results, setResults] = useState<NetworkDiagnosticResult[] | null>(null);

  const handleDiagnose = async () => {
    setDiagnosing(true);
    setResults(null);
    try {
      const data = await diagnoseNetwork();
      setResults(data);
    } catch (err: any) {
      setResults([
        {
          target: t('app.networkDiag.error'),
          reachable: false,
          response_time_ms: null,
          error: err?.message || t('app.networkDiag.failed'),
        },
      ]);
    } finally {
      setDiagnosing(false);
    }
  };

  const getSpeedLabel = (ms: number | null) => {
    if (ms === null) return '';
    if (ms < 100) return t('app.networkDiag.excellent');
    if (ms < 300) return t('app.networkDiag.good');
    if (ms < 1000) return t('app.networkDiag.fair');
    return t('app.networkDiag.poor');
  };

  const getSpeedColor = (ms: number | null) => {
    if (ms === null) return '#ff4d4f';
    if (ms < 100) return '#52c41a';
    if (ms < 300) return '#1890ff';
    if (ms < 1000) return '#faad14';
    return '#ff4d4f';
  };

  return (
    <Card
      title={
        <Space>
          <GlobalOutlined />
          {t('app.networkDiag.title')}
        </Space>
      }
      extra={
        <Button
          type="primary"
          icon={<ReloadOutlined spin={diagnosing} />}
          onClick={handleDiagnose}
          loading={diagnosing}
        >
          {t('app.networkDiag.runDiag')}
        </Button>
      }
      className="svl-network-diag-card"
    >
      {diagnosing && (
        <div className="svl-network-diag-loading">
          <Spin size="large" />
          <p>{t('app.networkDiag.running')}</p>
        </div>
      )}

      {!diagnosing && results && results.length > 0 && (
        <Space direction="vertical" style={{ width: '100%' }} size="middle">
          {results.map((result, index) => (
            <Card
              key={index}
              size="small"
              className={`svl-network-diag-result ${result.reachable ? 'svl-network-diag-success' : 'svl-network-diag-fail'}`}
            >
              <div className="svl-network-diag-item">
                <div className="svl-network-diag-header">
                  {result.reachable ? (
                    <CheckCircleOutlined className="svl-network-diag-icon-success" />
                  ) : (
                    <CloseCircleOutlined className="svl-network-diag-icon-fail" />
                  )}
                  <span className="svl-network-diag-target">{result.target}</span>
                </div>

                {result.reachable && result.response_time_ms !== null && (
                  <div className="svl-network-diag-stats">
                    <div className="svl-network-diag-speed">
                      <span className="svl-network-diag-label">
                        {t('app.networkDiag.responseTime')}:
                      </span>
                      <span className="svl-network-diag-value">
                        {result.response_time_ms} ms
                      </span>
                      <span
                        className="svl-network-diag-speed-label"
                        style={{ color: getSpeedColor(result.response_time_ms) }}
                      >
                        {getSpeedLabel(result.response_time_ms)}
                      </span>
                    </div>
                    <Progress
                      percent={Math.min(100, Math.max(0, 100 - result.response_time_ms / 10))}
                      strokeColor={getSpeedColor(result.response_time_ms)}
                      showInfo={false}
                      size="small"
                    />
                  </div>
                )}

                {result.error && (
                  <Alert
                    message={result.error}
                    type="error"
                    showIcon
                    className="svl-network-diag-error"
                  />
                )}
              </div>
            </Card>
          ))}

          {results.every(r => r.reachable) && (
            <Alert
              message={t('app.networkDiag.allGood')}
              description={t('app.networkDiag.allGoodDesc')}
              type="success"
              showIcon
            />
          )}

          {results.some(r => !r.reachable) && (
            <Alert
              message={t('app.networkDiag.someFailed')}
              description={t('app.networkDiag.someFailedDesc')}
              type="warning"
              showIcon
            />
          )}
        </Space>
      )}

      {!diagnosing && !results && (
        <div className="svl-network-diag-empty">
          <GlobalOutlined className="svl-network-diag-empty-icon" />
          <p>{t('app.networkDiag.emptyHint')}</p>
        </div>
      )}
    </Card>
  );
}