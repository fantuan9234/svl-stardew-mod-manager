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
  loadEditorQuests,
  saveEditorQuests,
  loadEditorBuildings,
  saveEditorBuildings,
  loadEditorFriendships,
  saveEditorFriendships,
  loadEditorRecipes,
  saveEditorRecipes,
  type SaveInfo,
  type SaveEditorSummary,
  type SaveEditorCharacterInfo,
  type SaveEditorSkillSet,
  type SaveEditorInventory,
  type SaveEditorItemInfo,
  type SaveEditorQuestLog,
  type SaveEditorQuestInfo,
  type SaveEditorBuildingList,
  type SaveEditorBuildingInfo,
  type SaveEditorFriendshipList,
  type SaveEditorFriendshipInfo,
  type SaveEditorRecipeData,
  type SaveEditorRecipeInfo,
} from '../utils/tauri-api';

export default function SaveEditorView({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [saves, setSaves] = useState<SaveInfo[]>([]);
  const [selectedSave, setSelectedSave] = useState<string | null>(null);
  const [summary, setSummary] = useState<SaveEditorSummary | null>(null);
  const [character, setCharacter] = useState<SaveEditorCharacterInfo | null>(null);
  const [skills, setSkills] = useState<SaveEditorSkillSet | null>(null);
  const [inventory, setInventory] = useState<SaveEditorInventory | null>(null);
  const [quests, setQuests] = useState<SaveEditorQuestLog | null>(null);
  const [buildings, setBuildings] = useState<SaveEditorBuildingList | null>(null);
  const [friendships, setFriendships] = useState<SaveEditorFriendshipList | null>(null);
  const [recipes, setRecipes] = useState<SaveEditorRecipeData | null>(null);
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
      const [s, c, sk, inv, q, b, f, r] = await Promise.all([
        openSaveInEditor(path),
        loadEditorCharacter(path),
        loadEditorSkills(path),
        loadEditorInventory(path),
        loadEditorQuests(path),
        loadEditorBuildings(path),
        loadEditorFriendships(path),
        loadEditorRecipes(path),
      ]);
      setSummary(s);
      setCharacter(c);
      setSkills(sk);
      setInventory(inv);
      setQuests(q);
      setBuildings(b);
      setFriendships(f);
      setRecipes(r);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.saveEditorLoadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleSaveAll = async () => {
    if (!selectedSave || !character || !skills || !inventory || !quests || !buildings) return;
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
          await saveEditorQuests(selectedSave, quests);
          await saveEditorBuildings(selectedSave, buildings);
          if (friendships) await saveEditorFriendships(selectedSave, friendships);
          if (recipes) await saveEditorRecipes(selectedSave, recipes);
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
        ) : !character || !skills || !inventory || !quests || !buildings ? null : (
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
                children: (
                  <QuestForm
                    value={quests!}
                    onChange={(v) => {
                      setQuests(v);
                      setDirty(true);
                    }}
                  />
                ),
              },
              {
                key: 'buildings',
                label: (
                  <span>
                    <HomeOutlined /> {t('app.toolbox.saveEditorTabBuildings')}
                  </span>
                ),
                children: (
                  <BuildingForm
                    value={buildings!}
                    onChange={(v) => {
                      setBuildings(v);
                      setDirty(true);
                    }}
                  />
                ),
              },
              {
                key: 'friendships',
                label: (
                  <span>
                    <UserOutlined /> {t('app.toolbox.saveEditorTabFriendships')}
                  </span>
                ),
                children: (
                  <FriendshipForm
                    value={friendships}
                    onChange={(v) => {
                      setFriendships(v);
                      setDirty(true);
                    }}
                  />
                ),
              },
              {
                key: 'recipes',
                label: (
                  <span>
                    <ToolOutlined /> {t('app.toolbox.saveEditorTabRecipes')}
                  </span>
                ),
                children: (
                  <RecipeForm
                    value={recipes}
                    onChange={(v) => {
                      setRecipes(v);
                      setDirty(true);
                    }}
                  />
                ),
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

function QuestForm({
  value,
  onChange,
}: {
  value: SaveEditorQuestLog;
  onChange: (v: SaveEditorQuestLog) => void;
}) {
  const { t } = useTranslation();
  const updateField = (i: number, patch: Partial<SaveEditorQuestInfo>) => {
    onChange({
      quests: value.quests.map((q, j) => (j === i ? { ...q, ...patch } : q)),
    });
  };

  const removeQuest = (i: number) => {
    onChange({ quests: value.quests.filter((_, j) => j !== i) });
  };

  const columns = [
    {
      title: t('app.toolbox.saveEditorQuestId'),
      dataIndex: 'id',
      width: 70,
      render: (val: string) => <code>{val}</code>,
    },
    {
      title: t('app.toolbox.saveEditorQuestTitle'),
      dataIndex: 'title',
      width: 200,
      render: (val: string) => val || '(无标题)',
    },
    {
      title: t('app.toolbox.saveEditorQuestObjective'),
      dataIndex: 'current_objective',
      render: (val: string) => val || '-',
    },
    {
      title: t('app.toolbox.saveEditorQuestReward'),
      dataIndex: 'money_reward',
      width: 100,
      render: (val: number, record: SaveEditorQuestInfo) => (
        <InputNumber
          size="small"
          min={0}
          value={val}
          onChange={(v) => updateField(record.index, { money_reward: v ?? 0 })}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorQuestDaysLeft'),
      dataIndex: 'days_left',
      width: 90,
      render: (val: number) => <code>{val}</code>,
    },
    {
      title: t('app.toolbox.saveEditorQuestCompleted'),
      dataIndex: 'completed',
      width: 100,
      render: (val: boolean, record: SaveEditorQuestInfo) => (
        <input
          type="checkbox"
          checked={val}
          onChange={(e) => updateField(record.index, { completed: e.target.checked })}
        />
      ),
    },
    {
      title: '',
      key: 'action',
      width: 60,
      render: (_: any, record: SaveEditorQuestInfo) => (
        <Popconfirm
          title={t('app.toolbox.saveEditorItemDelete')}
          onConfirm={() => removeQuest(record.index)}
        >
          <Button size="small" danger icon={<DeleteOutlined />} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <div>
      {value.quests.length === 0 ? (
        <Empty description={t('app.toolbox.saveEditorQuestEmpty')} />
      ) : (
        <Table
          rowKey="index"
          size="small"
          dataSource={value.quests}
          columns={columns}
          pagination={{ pageSize: 30, showSizeChanger: false }}
        />
      )}
    </div>
  );
}

function BuildingForm({
  value,
  onChange,
}: {
  value: SaveEditorBuildingList;
  onChange: (v: SaveEditorBuildingList) => void;
}) {
  const { t } = useTranslation();
  const updateField = (i: number, patch: Partial<SaveEditorBuildingInfo>) => {
    onChange({
      buildings: value.buildings.map((b, j) => (j === i ? { ...b, ...patch } : b)),
    });
  };

  const handleDelete = (i: number) => {
    onChange({
      buildings: value.buildings.filter((_, j) => j !== i).map((b, j) => ({ ...b, index: j })),
    });
  };

  const columns = [
    {
      title: t('app.toolbox.saveEditorBuildingLocation'),
      dataIndex: 'location',
      width: 130,
      render: (val: string) => <code>{val}</code>,
    },
    {
      title: t('app.toolbox.saveEditorBuildingType'),
      dataIndex: 'building_type',
      width: 110,
      render: (val: string) => <code>{val}</code>,
    },
    {
      title: t('app.toolbox.saveEditorBuildingTileX'),
      dataIndex: 'tile_x',
      width: 90,
      render: (val: number, record: SaveEditorBuildingInfo) => (
        <InputNumber
          size="small"
          value={val}
          onChange={(v) => updateField(record.index, { tile_x: v ?? 0 })}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorBuildingTileY'),
      dataIndex: 'tile_y',
      width: 90,
      render: (val: number, record: SaveEditorBuildingInfo) => (
        <InputNumber
          size="small"
          value={val}
          onChange={(v) => updateField(record.index, { tile_y: v ?? 0 })}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorBuildingMaxOccupants'),
      dataIndex: 'max_occupants',
      width: 100,
      render: (val: number) => <code>{val}</code>,
    },
    {
      title: t('app.toolbox.saveEditorBuildingCurrentOccupants'),
      dataIndex: 'current_occupants',
      width: 100,
      render: (val: number) => <code>{val}</code>,
    },
    {
      title: t('app.toolbox.saveEditorBuildingAction'),
      dataIndex: 'action',
      width: 80,
      fixed: 'right' as const,
      render: (_: unknown, record: SaveEditorBuildingInfo) => (
        <Popconfirm
          title={t('app.toolbox.saveEditorBuildingDeleteConfirm', '确定要删除这个建筑吗？')}
          okText={t('common.confirm', '确定')}
          cancelText={t('common.cancel', '取消')}
          onConfirm={() => handleDelete(record.index)}
        >
          <Button
            size="small"
            type="link"
            danger
            icon={<DeleteOutlined />}
          >
            {t('app.toolbox.saveEditorBuildingDelete', '删除')}
          </Button>
        </Popconfirm>
      ),
    },
  ];

  return (
    <div>
      {value.buildings.length === 0 ? (
        <Empty description={t('app.toolbox.saveEditorBuildingEmpty')} />
      ) : (
        <Table
          rowKey="index"
          size="small"
          dataSource={value.buildings}
          columns={columns}
          pagination={{ pageSize: 30, showSizeChanger: false }}
        />
      )}
    </div>
  );
}

function FriendshipForm({
  value,
  onChange,
}: {
  value: SaveEditorFriendshipList | null;
  onChange: (v: SaveEditorFriendshipList) => void;
}) {
  const { t } = useTranslation();

  if (!value) return <Empty description={t('app.toolbox.saveEditorBuildingEmpty')} />;

  const updateField = (i: number, patch: Partial<SaveEditorFriendshipInfo>) => {
    onChange({
      friendships: value.friendships.map((f, j) => (j === i ? { ...f, ...patch } : f)),
    });
  };

  const columns = [
    {
      title: t('app.toolbox.saveEditorFriendshipNpc'),
      dataIndex: 'npc_name',
      width: 120,
    },
    {
      title: t('app.toolbox.saveEditorFriendshipHearts'),
      dataIndex: 'points',
      width: 80,
      render: (val: number, record: SaveEditorFriendshipInfo) => {
        const hearts = Math.floor(val / 250);
        const maxHearts = record.status === 'Married' ? 14 : record.status === 'Dating' ? 11 : 10;
        return (
          <InputNumber
            min={0}
            max={maxHearts * 250}
            step={250}
            value={val}
            onChange={(v) => updateField(record.index, { points: v ?? 0 })}
            style={{ width: 80 }}
            addonAfter={`${hearts}/${maxHearts}❤`}
          />
        );
      },
    },
    {
      title: t('app.toolbox.saveEditorFriendshipStatus'),
      dataIndex: 'status',
      width: 80,
    },
    {
      title: t('app.toolbox.saveEditorFriendshipGiftsWeek'),
      dataIndex: 'gifts_this_week',
      width: 80,
      render: (val: number, record: SaveEditorFriendshipInfo) => (
        <InputNumber
          min={0}
          max={2}
          value={val}
          onChange={(v) => updateField(record.index, { gifts_this_week: v ?? 0 })}
          style={{ width: 60 }}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorFriendshipGiftsToday'),
      dataIndex: 'gifts_today',
      width: 80,
      render: (val: number, record: SaveEditorFriendshipInfo) => (
        <InputNumber
          min={0}
          max={2}
          value={val}
          onChange={(v) => updateField(record.index, { gifts_today: v ?? 0 })}
          style={{ width: 60 }}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorFriendshipTalked'),
      dataIndex: 'talked_to_today',
      width: 70,
      render: (val: boolean, record: SaveEditorFriendshipInfo) => (
        <input
          type="checkbox"
          checked={val}
          onChange={(e) => updateField(record.index, { talked_to_today: e.target.checked })}
        />
      ),
    },
  ];

  return (
    <div>
      {value.friendships.length === 0 ? (
        <Empty description={t('app.toolbox.saveEditorBuildingEmpty')} />
      ) : (
        <Table
          rowKey="index"
          size="small"
          dataSource={value.friendships}
          columns={columns}
          pagination={{ pageSize: 30, showSizeChanger: false }}
        />
      )}
    </div>
  );
}

function RecipeForm({
  value,
  onChange,
}: {
  value: SaveEditorRecipeData | null;
  onChange: (v: SaveEditorRecipeData) => void;
}) {
  const { t } = useTranslation();

  if (!value) return <Empty description={t('app.toolbox.saveEditorRecipeEmpty')} />;

  const updateCooking = (i: number, patch: Partial<SaveEditorRecipeInfo>) => {
    onChange({
      ...value,
      cooking: value.cooking.map((r, j) => (j === i ? { ...r, ...patch } : r)),
    });
  };

  const updateCrafting = (i: number, patch: Partial<SaveEditorRecipeInfo>) => {
    onChange({
      ...value,
      crafting: value.crafting.map((r, j) => (j === i ? { ...r, ...patch } : r)),
    });
  };

  const recipeColumns = (updateFn: (i: number, patch: Partial<SaveEditorRecipeInfo>) => void) => [
    {
      title: t('app.toolbox.saveEditorRecipeName'),
      dataIndex: 'name',
      width: 180,
    },
    {
      title: t('app.toolbox.saveEditorRecipeUnlocked'),
      dataIndex: 'unlocked',
      width: 80,
      render: (val: boolean, record: SaveEditorRecipeInfo) => (
        <input
          type="checkbox"
          checked={val}
          onChange={(e) => updateFn(record.index, { unlocked: e.target.checked })}
        />
      ),
    },
    {
      title: t('app.toolbox.saveEditorRecipeTimesCrafted'),
      dataIndex: 'times_crafted',
      width: 100,
      render: (val: number, record: SaveEditorRecipeInfo) => (
        <InputNumber
          min={0}
          value={val}
          onChange={(v) => updateFn(record.index, { times_crafted: v ?? 0 })}
          style={{ width: 80 }}
        />
      ),
    },
  ];

  return (
    <div>
      <h4>{t('app.toolbox.saveEditorCookingRecipes')}</h4>
      {value.cooking.length === 0 ? (
        <Empty description={t('app.toolbox.saveEditorRecipeEmpty')} />
      ) : (
        <Table
          rowKey="index"
          size="small"
          dataSource={value.cooking}
          columns={recipeColumns(updateCooking)}
          pagination={{ pageSize: 30, showSizeChanger: false }}
        />
      )}
      <h4 style={{ marginTop: 16 }}>{t('app.toolbox.saveEditorCraftingRecipes')}</h4>
      {value.crafting.length === 0 ? (
        <Empty description={t('app.toolbox.saveEditorRecipeEmpty')} />
      ) : (
        <Table
          rowKey="index"
          size="small"
          dataSource={value.crafting}
          columns={recipeColumns(updateCrafting)}
          pagination={{ pageSize: 30, showSizeChanger: false }}
        />
      )}
    </div>
  );
}
