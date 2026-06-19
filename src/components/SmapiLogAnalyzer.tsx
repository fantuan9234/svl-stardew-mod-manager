import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, Input, Empty, Tag, Space, Spin, Typography, Alert, message, Collapse, Tooltip } from 'antd';
import { ArrowLeftOutlined, ThunderboltOutlined, ClearOutlined, CopyOutlined, CheckOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';

const { Text, Paragraph, Title } = Typography;
const { TextArea } = Input;

interface LogError {
  raw_message: string;
  translated_message: string;
  severity: string;
  solution: string;
  solution_button_text: string;
}

interface LogWarning {
  raw_message: string;
  translated_message: string;
  suggestion: string;
}

interface PastedLogAnalysis {
  errors: LogError[];
  warnings: LogWarning[];
  error_count: number;
  warning_count: number;
  input_length: number;
  detected_issues: string[];
  suggestions: string[];
}

const SEVERITY_COLOR: Record<string, 'error' | 'warning' | 'info'> = {
  Error: 'error',
  Warning: 'warning',
  Info: 'info',
};

const EXAMPLE_LOGS: { key: string; content: string }[] = [
  {
    key: 'missingDep',
    content: `[20:52:18 ERROR SMAPI] These mods could not be added to your game.
[20:52:18 ERROR SMAPI] --------------------------------------------------
[20:52:18 ERROR SMAPI]    - Gifts from Linus 1.1.0 because it requires mods which aren't installed ('Mail Framework': https://www.nexusmods.com/stardewvalley/mods/1536).`,
  },
  {
    key: 'dllFailed',
    content: `[16:40:42 ERROR SMAPI] - UI Info Suite 2 2.0.0 because its DLL couldn't be loaded.
[16:40:42 ERROR SMAPI]    (Error: System.Exception: Rewriting UIInfoSuite2.dll failed.
 ---> Mono.Cecil.AssemblyResolutionException: Failed to resolve assembly: 'System.Windows.Extensions, Version=0.0.0.0'`,
  },
  {
    key: 'incompatible',
    content: `[16:40:42 ERROR SMAPI] - CJB Item Spawner 2.0.0 because it's no longer compatible`,
  },
];

export default function SmapiLogAnalyzer({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<PastedLogAnalysis | null>(null);
  const [copied, setCopied] = useState(false);

  const handleAnalyze = useCallback(async () => {
    if (!content.trim()) {
      message.warning(t('app.smapiAnalyzer.emptyInput'));
      return;
    }
    setLoading(true);
    try {
      const res = await invoke<PastedLogAnalysis>('analyze_pasted_smapi_log', { content });
      setResult(res);
      if (res.error_count === 0 && res.warning_count === 0 && res.detected_issues.length === 0) {
        message.info(t('app.smapiAnalyzer.noIssues'));
      } else {
        message.success(t('app.smapiAnalyzer.analyzeSuccess', { count: res.error_count + res.warning_count }));
      }
    } catch (e: any) {
      message.error(e?.toString() || t('app.smapiAnalyzer.analyzeFailed'));
    } finally {
      setLoading(false);
    }
  }, [content, t]);

  const handleClear = () => {
    setContent('');
    setResult(null);
  };

  const handleCopy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      message.error(t('app.smapiAnalyzer.copyFailed'));
    }
  };

  const loadExample = (key: string) => {
    const ex = EXAMPLE_LOGS.find(e => e.key === key);
    if (ex) {
      setContent(ex.content);
      setResult(null);
    }
  };

  return (
    <div style={{ padding: '24px 28px', maxWidth: 1100, margin: '0 auto' }}>
      <Space style={{ marginBottom: 16 }}>
        <Button icon={<ArrowLeftOutlined />} onClick={onBack}>
          {t('app.common.cancel')}
        </Button>
      </Space>

      <div style={{ marginBottom: 20 }}>
        <Title level={3} style={{ marginBottom: 6 }}>
          {t('app.toolbox.smapiAnalyzerTitle')}
        </Title>
        <Text type="secondary" style={{ fontSize: 13 }}>
          {t('app.toolbox.smapiAnalyzerDesc')}
        </Text>
      </div>

      <Alert
        type="info"
        showIcon
        style={{ marginBottom: 16, borderRadius: 8 }}
        message={t('app.smapiAnalyzer.hintTitle')}
        description={
          <div>
            <div>{t('app.smapiAnalyzer.hintStep1')}</div>
            <div>{t('app.smapiAnalyzer.hintStep2')}</div>
            <div>{t('app.smapiAnalyzer.hintStep3')}</div>
          </div>
        }
      />

      <div style={{
        background: 'var(--svl-bg-card)',
        border: '1px solid var(--svl-border)',
        borderRadius: 10,
        padding: 16,
        marginBottom: 16,
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <Text strong style={{ fontSize: 13 }}>
            {t('app.smapiAnalyzer.inputLabel')}
          </Text>
          <Space size={4}>
            <Text type="secondary" style={{ fontSize: 12 }}>{t('app.smapiAnalyzer.examples')}:</Text>
            <Button size="small" type="link" onClick={() => loadExample('missingDep')}>
              {t('app.smapiAnalyzer.exMissingDep')}
            </Button>
            <Button size="small" type="link" onClick={() => loadExample('dllFailed')}>
              {t('app.smapiAnalyzer.exDllFailed')}
            </Button>
            <Button size="small" type="link" onClick={() => loadExample('incompatible')}>
              {t('app.smapiAnalyzer.exIncompatible')}
            </Button>
          </Space>
        </div>
        <TextArea
          value={content}
          onChange={e => setContent(e.target.value)}
          placeholder={t('app.smapiAnalyzer.inputPlaceholder')}
          autoSize={{ minRows: 8, maxRows: 18 }}
          style={{
            fontFamily: 'Consolas, Monaco, monospace',
            fontSize: 12,
            background: 'var(--svl-bg-primary)',
          }}
          spellCheck={false}
        />
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: 12 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {t('app.smapiAnalyzer.charCount', { count: content.length })}
          </Text>
          <Space>
            <Button icon={<ClearOutlined />} onClick={handleClear} disabled={!content}>
              {t('app.smapiAnalyzer.clear')}
            </Button>
            <Button
              type="primary"
              icon={<ThunderboltOutlined />}
              loading={loading}
              onClick={handleAnalyze}
              disabled={!content.trim()}
            >
              {loading ? t('app.smapiAnalyzer.analyzing') : t('app.smapiAnalyzer.analyze')}
            </Button>
          </Space>
        </div>
      </div>

      {loading && (
        <div style={{ textAlign: 'center', padding: 40 }}>
          <Spin tip={t('app.smapiAnalyzer.analyzing')} />
        </div>
      )}

      {!loading && result && (
        <div>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
            gap: 10,
            marginBottom: 16,
          }}>
            <SummaryCard
              label={t('app.smapiAnalyzer.summaryErrors')}
              value={result.error_count}
              color="#ff4d4f"
              icon="❌"
            />
            <SummaryCard
              label={t('app.smapiAnalyzer.summaryWarnings')}
              value={result.warning_count}
              color="#faad14"
              icon="⚠️"
            />
            <SummaryCard
              label={t('app.smapiAnalyzer.summaryDetected')}
              value={result.detected_issues.length}
              color="#1890ff"
              icon="🔍"
            />
            <SummaryCard
              label={t('app.smapiAnalyzer.summarySuggestions')}
              value={result.suggestions.length}
              color="#52c41a"
              icon="💡"
            />
          </div>

          {result.detected_issues.length > 0 && (
            <div style={{
              background: 'var(--svl-bg-card)',
              border: '1px solid var(--svl-border)',
              borderRadius: 10,
              padding: 16,
              marginBottom: 14,
            }}>
              <Text strong style={{ fontSize: 14, display: 'block', marginBottom: 10 }}>
                {t('app.smapiAnalyzer.detectedTitle')}
              </Text>
              <Space direction="vertical" size={6} style={{ width: '100%' }}>
                {result.detected_issues.map((issue, i) => (
                  <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: 8 }}>
                    <Tag color="blue" style={{ margin: 0, flexShrink: 0 }}>{i + 1}</Tag>
                    <Text style={{ fontSize: 13 }}>{issue}</Text>
                  </div>
                ))}
              </Space>
            </div>
          )}

          {result.suggestions.length > 0 && (
            <Alert
              type="success"
              showIcon
              style={{ marginBottom: 14, borderRadius: 10 }}
              message={t('app.smapiAnalyzer.suggestionTitle')}
              description={
                <Space direction="vertical" size={6} style={{ width: '100%', marginTop: 4 }}>
                  {result.suggestions.map((s, i) => (
                    <div key={i} style={{ display: 'flex', alignItems: 'flex-start', gap: 8 }}>
                      <Text style={{ color: '#52c41a', fontWeight: 600 }}>✓</Text>
                      <Text style={{ fontSize: 13, lineHeight: 1.6 }}>{s}</Text>
                    </div>
                  ))}
                </Space>
              }
            />
          )}

          {result.errors.length > 0 && (
            <Collapse
              ghost
              defaultActiveKey={['errors']}
              style={{
                background: 'var(--svl-bg-card)',
                border: '1px solid var(--svl-border)',
                borderRadius: 10,
                marginBottom: 14,
                padding: '0 4px',
              }}
              items={[{
                key: 'errors',
                label: (
                  <Space>
                    <Text strong style={{ color: '#ff4d4f', fontSize: 14 }}>
                      {t('app.smapiAnalyzer.errorsTitle', { count: result.error_count })}
                    </Text>
                  </Space>
                ),
                children: (
                  <Space direction="vertical" size={10} style={{ width: '100%' }}>
                    {result.errors.map((err, i) => (
                      <div key={i} style={{
                        background: 'var(--svl-bg-primary)',
                        border: '1px solid var(--svl-border)',
                        borderLeft: '3px solid #ff4d4f',
                        borderRadius: 6,
                        padding: 12,
                      }}>
                        <Space style={{ marginBottom: 6 }} size={6} wrap>
                          <Tag color={SEVERITY_COLOR[err.severity] || 'error'}>{err.severity}</Tag>
                          <Tag>{err.translated_message}</Tag>
                        </Space>
                        <div style={{ marginBottom: 8 }}>
                          <Text type="secondary" style={{ fontSize: 11, display: 'block', marginBottom: 2 }}>
                            {t('app.smapiAnalyzer.rawLine')}
                          </Text>
                          <Paragraph
                            code
                            copyable
                            style={{
                              fontSize: 11,
                              padding: '6px 8px',
                              background: 'rgba(0,0,0,0.04)',
                              borderRadius: 4,
                              margin: 0,
                              wordBreak: 'break-all',
                            }}
                          >
                            {err.raw_message}
                          </Paragraph>
                        </div>
                        <div>
                          <Text type="secondary" style={{ fontSize: 11, display: 'block', marginBottom: 2 }}>
                            {t('app.smapiAnalyzer.solution')}
                          </Text>
                          <Text style={{ fontSize: 13, lineHeight: 1.6, whiteSpace: 'pre-wrap' }}>
                            {err.solution}
                          </Text>
                        </div>
                      </div>
                    ))}
                  </Space>
                ),
              }]}
            />
          )}

          {result.warnings.length > 0 && (
            <Collapse
              ghost
              style={{
                background: 'var(--svl-bg-card)',
                border: '1px solid var(--svl-border)',
                borderRadius: 10,
                marginBottom: 14,
                padding: '0 4px',
              }}
              items={[{
                key: 'warnings',
                label: (
                  <Text strong style={{ color: '#faad14', fontSize: 14 }}>
                    {t('app.smapiAnalyzer.warningsTitle', { count: result.warning_count })}
                  </Text>
                ),
                children: (
                  <Space direction="vertical" size={10} style={{ width: '100%' }}>
                    {result.warnings.map((w, i) => (
                      <div key={i} style={{
                        background: 'var(--svl-bg-primary)',
                        border: '1px solid var(--svl-border)',
                        borderLeft: '3px solid #faad14',
                        borderRadius: 6,
                        padding: 12,
                      }}>
                        <Space style={{ marginBottom: 6 }} size={6} wrap>
                          <Tag color="warning">Warning</Tag>
                          <Tag>{w.translated_message}</Tag>
                        </Space>
                        <Paragraph
                          code
                          style={{
                            fontSize: 11,
                            padding: '6px 8px',
                            background: 'rgba(0,0,0,0.04)',
                            borderRadius: 4,
                            margin: '6px 0',
                            wordBreak: 'break-all',
                          }}
                        >
                          {w.raw_message}
                        </Paragraph>
                        <Text style={{ fontSize: 13, lineHeight: 1.6 }}>{w.suggestion}</Text>
                      </div>
                    ))}
                  </Space>
                ),
              }]}
            />
          )}

          {result.error_count === 0 && result.warning_count === 0 && result.detected_issues.length === 0 && (
            <Empty
              description={t('app.smapiAnalyzer.noIssues')}
              style={{ padding: 40 }}
            />
          )}

          <div style={{ textAlign: 'center', marginTop: 16 }}>
            <Tooltip title={copied ? t('app.smapiAnalyzer.copied') : t('app.smapiAnalyzer.copySummary')}>
              <Button
                icon={copied ? <CheckOutlined /> : <CopyOutlined />}
                onClick={() => {
                  const summary = [
                    `=== SVL SMAPI 日志检测结果 ===`,
                    `输入长度: ${result.input_length} 字符`,
                    `错误数: ${result.error_count}, 警告数: ${result.warning_count}`,
                    ``,
                    `--- 检测到的问题 ---`,
                    ...result.detected_issues.map((s, i) => `${i + 1}. ${s}`),
                    ``,
                    `--- 建议 ---`,
                    ...result.suggestions.map((s, i) => `${i + 1}. ${s}`),
                  ].join('\n');
                  handleCopy(summary);
                }}
              >
                {t('app.smapiAnalyzer.copySummary')}
              </Button>
            </Tooltip>
          </div>
        </div>
      )}
    </div>
  );
}

function SummaryCard({ label, value, color, icon }: { label: string; value: number; color: string; icon: string }) {
  return (
    <div style={{
      background: 'var(--svl-bg-card)',
      border: '1px solid var(--svl-border)',
      borderRadius: 10,
      padding: '14px 16px',
      textAlign: 'center',
    }}>
      <div style={{ fontSize: 18, marginBottom: 4 }}>{icon}</div>
      <div style={{ fontSize: 22, fontWeight: 700, color, lineHeight: 1.2 }}>{value}</div>
      <div style={{ fontSize: 11, color: 'var(--svl-text-secondary)', marginTop: 2 }}>{label}</div>
    </div>
  );
}
