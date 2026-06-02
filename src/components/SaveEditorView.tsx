import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Card, Button, Form, Input, InputNumber, Tabs, message, Modal, Spin, Empty, Select, Space, Table, Popconfirm, Row, Col } from 'antd';
import { ArrowLeftOutlined, SaveOutlined, UserOutlined, ToolOutlined, ShoppingOutlined, TrophyOutlined, HomeOutlined, PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import {
  scanSaves,
  openSaveInEditor,
  loadEditorCharacter,
  saveEditorCharacter,
  loadEditorSkills,
  saveEditorSkills,
  loadEditorInventory,
  saveEditorInventory,
  type SaveInfo,
  type SaveEditorSummary,
  type SaveEditorCharacterInfo,
  type SaveEditorSkillSet,
  type SaveEditorInventory,
  type SaveEditorItemInfo,
} from '../utils/tauri-api';

export default function SaveEditorView({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [saves, setSaves] = useState<SaveInfo[]>([]);
  const [selectedSave, setSelectedSave] = useState<string | null>(null);
  const [summary, setSummary] = useState<SaveEditorSummary | null>(null);
  const [character, setCharacter] = useState<SaveEditorCharacterInfo | null>(null);
  const [skills, setSkills] = useState<SaveEditorSkillSet | null>(null);
  const [inventory, setInventory] = useState<SaveEditorInventory | null>(null);
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
      const [s, c, sk, inv] = await Promise.all([
        openSaveInEditor(path),
        loadEditorCharacter(path),
        loadEditorSkills(path),
        loadEditorInventory(path),
      ]);
      setSummary(s);
      setCharacter(c);
      setSkills(sk);
      setInventory(inv);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.saveEditorLoadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleSaveAll = async () => {
    if (!selectedSave || !character || !skills || !inventory) return;
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
          await saveEditorInventory(selectedSave, inventory);
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
        ) : !character || !skills || !inventory ? null : (
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
                children: (
                  <InventoryForm
                    value={inventory!}
                    onChange={(v) => {
                      setInventory(v);
                      setDirty(true);
                    }}
                  />
                ),
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
      <Form.Item label={t('app.toolbox.saveEditorDay')}>
        <InputNumber
          value={value.day_of_month}
          min={1}
          max={28}
          style={{ width: '100%' }}
          onChange={(v) => onChange({ ...value, day_of_month: v ?? 1 })}
        />
      </Form.Item>
      <Form.Item label={t('app.toolbox.saveEditorSeason')}>
        <Select
          value={value.current_season}
          style={{ width: '100%' }}
          onChange={(v) => onChange({ ...value, current_season: v })}
          options={[
            { value: 'spring', label: t('app.toolbox.saveEditorSpring') },
            { value: 'summer', label: t('app.toolbox.saveEditorSummer') },
            { value: 'fall', label: t('app.toolbox.saveEditorFall') },
            { value: 'winter', label: t('app.toolbox.saveEditorWinter') },
          ]}
        />
      </Form.Item>
      <Form.Item label={t('app.toolbox.saveEditorYear')}>
        <InputNumber
          value={value.year}
          min={1}
          max={999}
          style={{ width: '100%' }}
          onChange={(v) => onChange({ ...value, year: v ?? 1 })}
        />
      </Form.Item>
      <Form.Item label={t('app.toolbox.saveEditorTime')}>
        <InputNumber
          value={value.time_of_day}
          min={0}
          max={2400}
          step={10}
          style={{ width: '100%' }}
          onChange={(v) => onChange({ ...value, time_of_day: v ?? 600 })}
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

const COMMON_ITEMS: { id: number; name: string }[] = [
  { id: 0, name: 'Parsnip' },
  { id: 16, name: 'Melon' },
  { id: 24, name: 'Parsnip Seeds' },
  { id: 245, name: 'Wheat Seeds' },
  { id: 286, name: 'Diamond' },
  { id: 334, name: 'Copper Bar' },
  { id: 335, name: 'Iron Bar' },
  { id: 336, name: 'Gold Bar' },
  { id: 337, name: 'Iridium Bar' },
  { id: 390, name: 'Stone' },
  { id: 388, name: 'Wood' },
  { id: 382, name: 'Coal' },
  { id: 709, name: 'Prismatic Shard' },
  { id: 74, name: 'Prismatic Shard (old)' },
];

function InventoryForm({
  value,
  onChange,
}: {
  value: SaveEditorInventory;
  onChange: (v: SaveEditorInventory) => void;
}) {
  const { t } = useTranslation();
  const [newId, setNewId] = useState<number>(0);
  const [newStack, setNewStack] = useState<number>(1);
  const [newName, setNewName] = useState<string>('');

  const updateStack = (index: number, stack: number) => {
    onChange({
      items: value.items.map((it) => (it.index === index ? { ...it, stack } : it)),
    });
  };

  const updateName = (index: number, name: string) => {
    onChange({
      items: value.items.map((it) => (it.index === index ? { ...it, name } : it)),
    });
  };

  const removeItem = (index: number) => {
    onChange({ items: value.items.filter((it) => it.index !== index) });
  };

  const addItem = () => {
    const newIndex = value.items.length > 0
      ? Math.max(...value.items.map((i) => i.index)) + 1
      : 0;
    const nameToUse = newName || COMMON_ITEMS.find((c) => c.id === newId)?.name || '';
    onChange({
      items: [
        ...value.items,
        {
          index: newIndex,
          item_id: newId,
          stack: newStack,
          name: nameToUse,
          quality: 0,
          raw_xml: '',
        },
      ],
    });
    setNewName('');
  };

  const columns = [
    {
      title: t('app.toolbox.saveEditorItemId'),
      dataIndex: 'item_id',
      width: 100,
      render: (val: number) => <code>{val}</code>,
    },
    {
      title: t('app.toolbox.saveEditorItemName'),
      dataIndex: 'name',
      render: (val: string, record: SaveEditorItemInfo) => (
        <Input
          size="small"
          value={val}
          onChange={(e) => updateName(record.index, e.target.value)}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorItemStack'),
      dataIndex: 'stack',
      width: 120,
      render: (val: number, record: SaveEditorItemInfo) => (
        <InputNumber
          size="small"
          min={0}
          value={val}
          onChange={(v) => updateStack(record.index, v ?? 0)}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorItemQuality'),
      dataIndex: 'quality',
      width: 80,
      render: (val: number) => <code>{val}</code>,
    },
    {
      title: '',
      key: 'action',
      width: 80,
      render: (_: any, record: SaveEditorItemInfo) => (
        <Popconfirm
          title={t('app.toolbox.saveEditorItemDelete')}
          onConfirm={() => removeItem(record.index)}
        >
          <Button size="small" danger icon={<DeleteOutlined />} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <div>
      <Card size="small" style={{ marginBottom: 12 }} title={t('app.toolbox.saveEditorItemAdd')}>
        <Row gutter={8}>
          <Col span={8}>
            <Select
              style={{ width: '100%' }}
              value={newId}
              onChange={setNewId}
              options={COMMON_ITEMS.map((c) => ({
                value: c.id,
                label: `${c.id} - ${c.name}`,
              }))}
              showSearch
              optionFilterProp="label"
              placeholder={t('app.toolbox.saveEditorCommonItems')}
            />
          </Col>
          <Col span={4}>
            <InputNumber
              style={{ width: '100%' }}
              min={0}
              max={999}
              value={newStack}
              onChange={(v) => setNewStack(v ?? 1)}
            />
          </Col>
          <Col span={8}>
            <Input
              placeholder={t('app.toolbox.saveEditorItemName')}
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
            />
          </Col>
          <Col span={4}>
            <Button block type="primary" icon={<PlusOutlined />} onClick={addItem}>
              {t('app.toolbox.saveEditorItemAdd')}
            </Button>
          </Col>
        </Row>
      </Card>
      {value.items.length === 0 ? (
        <Empty description={t('app.toolbox.saveEditorItemEmpty')} />
      ) : (
        <Table
          rowKey="index"
          size="small"
          dataSource={value.items}
          columns={columns}
          pagination={{ pageSize: 50, showSizeChanger: false }}
        />
      )}
    </div>
  );
}
