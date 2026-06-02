import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Card, Button, Form, Input, InputNumber, Tabs, message, Modal, Spin, Empty, Select, Space } from 'antd';
import { ArrowLeftOutlined, SaveOutlined, UserOutlined, ToolOutlined, ShoppingOutlined, TrophyOutlined, HomeOutlined } from '@ant-design/icons';
import {
  scanSaves,
  openSaveInEditor,
  loadEditorCharacter,
  saveEditorCharacter,
  loadEditorSkills,
  saveEditorSkills,
  type SaveInfo,
  type SaveEditorSummary,
  type SaveEditorCharacterInfo,
  type SaveEditorSkillSet,
} from '../utils/tauri-api';

export default function SaveEditorView({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [saves, setSaves] = useState<SaveInfo[]>([]);
  const [selectedSave, setSelectedSave] = useState<string | null>(null);
  const [summary, setSummary] = useState<SaveEditorSummary | null>(null);
  const [character, setCharacter] = useState<SaveEditorCharacterInfo | null>(null);
  const [skills, setSkills] = useState<SaveEditorSkillSet | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    scanSaves()
      .then(setSaves)
      .catch(() => message.error(t('app.saves.loadFailed')));
  }, [t]);

  const handleSelectSave = async (path: string) => {
    setSelectedSave(path);
    setLoading(true);
    setDirty(false);
    try {
      const [s, c, sk] = await Promise.all([
        openSaveInEditor(path),
        loadEditorCharacter(path),
        loadEditorSkills(path),
      ]);
      setSummary(s);
      setCharacter(c);
      setSkills(sk);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.saveEditorLoadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleSaveAll = async () => {
    if (!selectedSave || !character || !skills) return;
    Modal.confirm({
      title: t('app.toolbox.saveEditorConfirmTitle'),
      content: t('app.toolbox.saveEditorConfirmContent'),
      okText: t('app.common.ok'),
      cancelText: t('app.common.cancel'),
      onOk: async () => {
        setSaving(true);
        try {
          await saveEditorCharacter(selectedSave, character);
          await saveEditorSkills(selectedSave, skills);
          message.success(t('app.toolbox.saveEditorSaveSuccess'));
          setDirty(false);
        } catch (e: any) {
          message.error(e?.toString() || t('app.toolbox.saveEditorSaveFailed'));
        } finally {
          setSaving(false);
        }
      },
    });
  };

  return (
    <div className="svl-content">
      <div className="svl-save-editor">
        <div className="svl-save-editor-header">
          <Button icon={<ArrowLeftOutlined />} onClick={onBack}>
            {t('app.common.cancel')}
          </Button>
          <h2 style={{ margin: 0 }}>{t('app.toolbox.saveEditorTitle')}</h2>
          {selectedSave && (
            <Button
              type="primary"
              icon={<SaveOutlined />}
              loading={saving}
              onClick={handleSaveAll}
              disabled={!dirty}
            >
              {t('app.toolbox.saveEditorSave')}
            </Button>
          )}
        </div>

        <Card style={{ marginBottom: 16 }}>
          <Space wrap>
            <span>{t('app.toolbox.saveEditorSelectSave')}:</span>
            <Select
              style={{ minWidth: 320 }}
              value={selectedSave}
              onChange={handleSelectSave}
              loading={loading}
              options={saves.map((s) => ({
                value: s.save_path,
                label: `${s.character_name} - ${s.farm_name}`,
              }))}
              placeholder={t('app.toolbox.saveEditorSelectSave')}
            />
            {summary && (
              <span style={{ color: 'var(--svl-text-secondary)' }}>
                {summary.current_date}
              </span>
            )}
          </Space>
        </Card>

        {loading ? (
          <Spin />
        ) : !selectedSave ? (
          <Empty description={t('app.toolbox.saveEditorSelectSave')} />
        ) : !character || !skills ? null : (
          <Tabs
            items={[
              {
                key: 'character',
                label: (
                  <span>
                    <UserOutlined /> {t('app.toolbox.saveEditorTabCharacter')}
                  </span>
                ),
                children: (
                  <CharacterForm
                    value={character}
                    onChange={(v) => {
                      setCharacter(v);
                      setDirty(true);
                    }}
                  />
                ),
              },
              {
                key: 'skills',
                label: (
                  <span>
                    <ToolOutlined /> {t('app.toolbox.saveEditorTabSkills')}
                  </span>
                ),
                children: (
                  <SkillForm
                    value={skills}
                    onChange={(v) => {
                      setSkills(v);
                      setDirty(true);
                    }}
                  />
                ),
              },
              {
                key: 'inventory',
                label: (
                  <span>
                    <ShoppingOutlined /> {t('app.toolbox.saveEditorTabInventory')}
                  </span>
                ),
                children: <Empty description={t('app.toolbox.saveEditorNotImplemented')} />,
              },
              {
                key: 'quests',
                label: (
                  <span>
                    <TrophyOutlined /> {t('app.toolbox.saveEditorTabQuests')}
                  </span>
                ),
                children: <Empty description={t('app.toolbox.saveEditorNotImplemented')} />,
              },
              {
                key: 'buildings',
                label: (
                  <span>
                    <HomeOutlined /> {t('app.toolbox.saveEditorTabBuildings')}
                  </span>
                ),
                children: <Empty description={t('app.toolbox.saveEditorNotImplemented')} />,
              },
            ]}
          />
        )}
      </div>
    </div>
  );
}

function CharacterForm({
  value,
  onChange,
}: {
  value: SaveEditorCharacterInfo;
  onChange: (v: SaveEditorCharacterInfo) => void;
}) {
  const { t } = useTranslation();
  return (
    <Form layout="vertical" style={{ maxWidth: 480 }}>
      <Form.Item label={t('app.toolbox.saveEditorName')}>
        <Input
          value={value.name}
          onChange={(e) => onChange({ ...value, name: e.target.value })}
        />
      </Form.Item>
      <Form.Item label={t('app.toolbox.saveEditorFarmName')}>
        <Input
          value={value.farm_name}
          onChange={(e) => onChange({ ...value, farm_name: e.target.value })}
        />
      </Form.Item>
      <Form.Item label={t('app.toolbox.saveEditorMoney')}>
        <InputNumber
          value={value.money}
          min={0}
          style={{ width: '100%' }}
          onChange={(v) => onChange({ ...value, money: v ?? 0 })}
        />
      </Form.Item>
      <Form.Item label={t('app.toolbox.saveEditorHealth')}>
        <InputNumber
          value={value.health}
          min={0}
          max={value.max_health}
          style={{ width: '100%' }}
          onChange={(v) => onChange({ ...value, health: v ?? 0 })}
        />
      </Form.Item>
      <Form.Item label={t('app.toolbox.saveEditorStamina')}>
        <InputNumber
          value={value.stamina}
          min={0}
          max={value.max_stamina}
          style={{ width: '100%' }}
          onChange={(v) => onChange({ ...value, stamina: v ?? 0 })}
        />
      </Form.Item>
    </Form>
  );
}

function SkillForm({
  value,
  onChange,
}: {
  value: SaveEditorSkillSet;
  onChange: (v: SaveEditorSkillSet) => void;
}) {
  const updateSkill = (i: number, patch: Partial<{ level: number; experience: number }>) => {
    onChange({
      skills: value.skills.map((x, j) => (j === i ? { ...x, ...patch } : x)),
    });
  };

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
      {value.skills.map((s, i) => (
        <Card key={s.name} title={s.name} size="small">
          <Form layout="vertical">
            <Form.Item label="Level">
              <InputNumber
                min={0}
                max={10}
                value={s.level}
                style={{ width: '100%' }}
                onChange={(v) => updateSkill(i, { level: v ?? 0 })}
              />
            </Form.Item>
            <Form.Item label="Experience">
              <InputNumber
                min={0}
                value={s.experience}
                style={{ width: '100%' }}
                onChange={(v) => updateSkill(i, { experience: v ?? 0 })}
              />
            </Form.Item>
          </Form>
        </Card>
      ))}
    </div>
  );
}
