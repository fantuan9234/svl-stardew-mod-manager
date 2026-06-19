import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Card, Button, Tag, Modal, message, Empty, Select, List, Popconfirm, Tooltip, Collapse, Skeleton, Statistic, Row, Col, Progress, Pagination } from 'antd';
import {
  FolderOpenOutlined,
  SaveOutlined,
  UndoOutlined,
  LinkOutlined,
  DisconnectOutlined,
  ClockCircleOutlined,
  DatabaseOutlined,
  PlayCircleOutlined,
  UserOutlined,
  HomeOutlined,
  LockOutlined,
  HeartOutlined,
  TrophyOutlined,
  CalendarOutlined,
  DollarOutlined,
  ThunderboltOutlined,
  FireOutlined,
  GlobalOutlined,
  StarOutlined,
  ShoppingOutlined,
  BookOutlined,
  ToolOutlined,
  CrownOutlined,
  GiftOutlined,
  DownCircleOutlined,
  UpCircleOutlined,
} from '@ant-design/icons';
import { open } from '@tauri-apps/plugin-dialog';
import {
  scanSaves,
  getSaveDetails,
  backupSave,
  restoreSave,
  listSaveBackups,
  linkSaveToProfile,
  unlinkSaveFromProfile,
  launchGameWithSaveProfile,
  openSaveLocation,
  openBackupDialog,
  profileList,
  profileGetActive,
  checkSmapiStatus,
  scanProfileMods,
  type SaveInfo,
  type SaveDetailedInfo,
  type BackupInfo,
  type ProfileListItem,
  type SmapiInfo,
  type ProfileModInfo,
} from '../utils/tauri-api';

const FARM_TYPE_COLORS: Record<string, string> = {
  Standard: 'green',
  Riverland: 'blue',
  Forest: 'purple',
  'Hill-top': 'orange',
  Wilderness: 'red',
  'Four Corners': 'magenta',
  Beach: 'cyan',
  Custom: 'gold',
};

const FARM_TYPE_NAMES_I18N: Record<string, string> = {
  Standard: 'app.saves.farmTypeStandard',
  Riverland: 'app.saves.farmTypeRiverland',
  Forest: 'app.saves.farmTypeForest',
  'Hill-top': 'app.saves.farmTypeHilltop',
  Wilderness: 'app.saves.farmTypeWilderness',
  'Four Corners': 'app.saves.farmTypeFourCorners',
  Beach: 'app.saves.farmTypeBeach',
  Custom: 'app.saves.farmTypeCustom',
};

const SEASON_ICONS: Record<string, string> = {
  spring: '🌸',
  summer: '☀️',
  fall: '🍂',
  winter: '❄️',
};

function formatTimeOfDay(time: number): string {
  const hour = Math.floor(time / 100);
  const min = time % 100;
  return `${hour.toString().padStart(2, '0')}:${min.toString().padStart(2, '0')}`;
}

function formatMoney(amount: number): string {
  if (amount >= 1000000) return `${(amount / 1000000).toFixed(2)}M`;
  if (amount >= 10000) return `${(amount / 1000).toFixed(1)}K`;
  return amount.toLocaleString();
}

export default function SavesManager() {
  const { t } = useTranslation();
  const [gamePath, setGamePath] = useState<string>('');
  const [saves, setSaves] = useState<SaveInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [selectedSave, setSelectedSave] = useState<SaveInfo | null>(null);
  const [showBackupModal, setShowBackupModal] = useState(false);
  const [showRestoreModal, setShowRestoreModal] = useState(false);
  const [showDetailsModal, setShowDetailsModal] = useState(false);
  const [backingUp, setBackingUp] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [profiles, setProfiles] = useState<ProfileListItem[]>([]);
  const [profilesLoaded, setProfilesLoaded] = useState(false);
  const [launchingSave, setLaunchingSave] = useState<string | null>(null);
  const [changingProfile, setChangingProfile] = useState<string | null>(null);
  const [activeProfileName, setActiveProfileName] = useState<string | null>(null);
  const [requiredMods, setRequiredMods] = useState<ProfileModInfo[]>([]);
  const [expandedKeys, setExpandedKeys] = useState<string[]>([]);
  const [detailsCache, setDetailsCache] = useState<Record<string, SaveDetailedInfo>>({});
  const [detailsLoading, setDetailsLoading] = useState<Record<string, boolean>>({});
  const [currentPage, setCurrentPage] = useState(1);
  const pageSize = 4;

  useEffect(() => {
    checkSmapiStatus()
      .then((info: SmapiInfo) => {
        if (info.game_path) {
          setGamePath(info.game_path);
        }
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    loadSaves();
  }, []);

  useEffect(() => {
    if (gamePath && !profilesLoaded) {
      loadProfiles();
      loadActiveProfile();
      loadRequiredMods();
    }
  }, [gamePath]);

  const loadRequiredMods = async () => {
    if (!gamePath) return;
    try {
      const allMods = await scanProfileMods(gamePath);
      const required = allMods.filter((mod: ProfileModInfo) => mod.is_required);
      setRequiredMods(required);
    } catch {
    }
  };

  const loadActiveProfile = async () => {
    if (!gamePath) return;
    try {
      const activeName = await profileGetActive(gamePath);
      setActiveProfileName(activeName);
    } catch {
      setActiveProfileName(null);
    }
  };

  const loadSaves = async () => {
    setLoading(true);
    try {
      const list = await scanSaves();
      setSaves(list);
      setCurrentPage(1);
    } catch {
      message.error(t('app.saves.loadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const loadProfiles = async () => {
    if (!gamePath) return;
    try {
      const list = await profileList(gamePath);
      setProfiles(list);
      setProfilesLoaded(true);
    } catch {
      message.error(t('app.saves.loadProfilesFailed'));
    }
  };

  const loadDetailsFor = async (savePath: string) => {
    if (detailsCache[savePath] || detailsLoading[savePath]) return;
    setDetailsLoading((prev) => ({ ...prev, [savePath]: true }));
    try {
      const details = await getSaveDetails(savePath);
      setDetailsCache((prev) => ({ ...prev, [savePath]: details }));
    } catch (err: any) {
      message.error(t('app.saves.detailsLoadFailed') + ': ' + (err?.toString() || ''));
    } finally {
      setDetailsLoading((prev) => ({ ...prev, [savePath]: false }));
    }
  };

  const handleExpandChange = (keys: string[]) => {
    setExpandedKeys(keys);
    const newlyExpanded = keys.filter((k) => !expandedKeys.includes(k));
    newlyExpanded.forEach((savePath) => {
      loadDetailsFor(savePath);
    });
  };

  const handleToggleAll = () => {
    if (expandedKeys.length === saves.length) {
      setExpandedKeys([]);
    } else {
      setExpandedKeys(saves.map((s) => s.save_path));
      saves.forEach((s) => loadDetailsFor(s.save_path));
    }
  };

  const handleOpenSaveLocation = async () => {
    try {
      await openSaveLocation();
    } catch {
      message.error(t('app.saves.openLocationFailed'));
    }
  };

  const handleBackup = async (save: SaveInfo) => {
    setSelectedSave(save);
    setShowBackupModal(true);
  };

  const doBackup = async () => {
    if (!selectedSave) return;

    const backupDir = await open({
      title: t('app.saves.selectBackupDir'),
      directory: true,
    });

    if (!backupDir) return;

    setBackingUp(true);
    try {
      const result = await backupSave(selectedSave.save_path, backupDir as string);
      message.success(result.message);
      setShowBackupModal(false);
      loadSaves();
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.backupFailed'));
    } finally {
      setBackingUp(false);
    }
  };

  const handleRestore = async (save: SaveInfo) => {
    setSelectedSave(save);
    try {
      const backups = await listSaveBackups(save.save_path);
      setBackups(backups);
      setShowRestoreModal(true);
    } catch {
      message.error(t('app.saves.listBackupsFailed'));
    }
  };

  const doRestore = async (backup: BackupInfo) => {
    if (!selectedSave) return;

    setRestoring(true);
    try {
      const savesDir = await openBackupDialog();
      const result = await restoreSave(backup.backup_path, savesDir);
      message.success(result.message);
      setShowRestoreModal(false);
      loadSaves();
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.restoreFailed'));
    } finally {
      setRestoring(false);
    }
  };

  const handleProfileChange = async (save: SaveInfo, profileName: string) => {
    setChangingProfile(save.save_path);
    try {
      if (profileName === '__none__') {
        await unlinkSaveFromProfile(save.save_path);
        message.success(t('app.saves.unlinkSuccess'));
      } else {
        await linkSaveToProfile(save.save_path, profileName);
        message.success(t('app.saves.linkSuccess'));
      }
      loadSaves();
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.linkFailed'));
    } finally {
      setChangingProfile(null);
    }
  };

  const handleUnlink = async (save: SaveInfo) => {
    try {
      await unlinkSaveFromProfile(save.save_path);
      message.success(t('app.saves.unlinkSuccess'));
      loadSaves();
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.unlinkFailed'));
    }
  };

  const handleLaunchWithProfile = async (save: SaveInfo) => {
    if (!gamePath) {
      message.error(t('app.errors.gamePathNotFound'));
      return;
    }

    setLaunchingSave(save.save_path);
    try {
      const result = await launchGameWithSaveProfile(gamePath, save.save_path);
      if (result.success) {
        message.success(result.message);
      } else {
        message.error(result.message);
      }
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.launchFailed'));
    } finally {
      setLaunchingSave(null);
    }
  };

  const formatHours = (hours: number) => {
    if (hours >= 1) {
      const h = Math.floor(hours);
      const m = Math.round((hours - h) * 60);
      return m > 0 ? `${h}h ${m}m` : `${h}h`;
    }
    return `${Math.round(hours * 60)}m`;
  };

  const getProfileSelectValue = (save: SaveInfo) => {
    if (save.linked_profile) {
      const found = profiles.find(p => p.name === save.linked_profile);
      if (found) return found.name;
    }
    return '__none__';
  };

  const getFarmTypeLabel = (save: SaveInfo) => {
    const key = FARM_TYPE_NAMES_I18N[save.farm_type] || 'app.saves.farmTypeStandard';
    return t(key);
  };

  const renderSaveHeader = (save: SaveInfo) => {
    return (
      <div className="svl-save-card-header">
        <div className="svl-save-card-title">
          <UserOutlined /> {save.character_name || save.name}
        </div>
        <div className="svl-save-card-tags">
          <Tag color={FARM_TYPE_COLORS[save.farm_type] || 'default'}>
            {getFarmTypeLabel(save)}
          </Tag>
          {save.game_version && (
            <Tag className="svl-tag-info" style={{ fontSize: 11 }}>
              v{save.game_version}
            </Tag>
          )}
        </div>
      </div>
    );
  };

  const renderSaveSummary = (save: SaveInfo) => {
    return (
      <div className="svl-save-info">
        <Row gutter={[8, 4]}>
          <Col span={12}>
            <div className="svl-save-info-row">
              <span className="svl-save-info-label">
                <HomeOutlined style={{ marginRight: 4 }} />
                {t('app.saves.farmName')}:
              </span>
              <span className="svl-save-info-value">{save.farm_name || save.name}</span>
            </div>
          </Col>
          <Col span={12}>
            <div className="svl-save-info-row">
              <span className="svl-save-info-label">
                <CalendarOutlined style={{ marginRight: 4 }} />
                {t('app.saves.currentDate')}:
              </span>
              <span className="svl-save-info-value">
                {SEASON_ICONS[save.current_season] || ''} Y{save.year} {save.current_season} D{save.day_of_month}
              </span>
            </div>
          </Col>
          <Col span={12}>
            <div className="svl-save-info-row">
              <span className="svl-save-info-label">
                <DollarOutlined style={{ marginRight: 4 }} />
                {t('app.saves.money')}:
              </span>
              <span className="svl-save-info-value svl-save-money">{formatMoney(save.money)}g</span>
            </div>
          </Col>
          <Col span={12}>
            <div className="svl-save-info-row">
              <span className="svl-save-info-label">
                <DatabaseOutlined style={{ marginRight: 4 }} />
                {t('app.saves.fileSize')}:
              </span>
              <span className="svl-save-info-value">{save.file_size_mb.toFixed(2)} MB</span>
            </div>
          </Col>
        </Row>
      </div>
    );
  };

  const renderDetails = (save: SaveInfo) => {
    const details = detailsCache[save.save_path];
    const isLoading = detailsLoading[save.save_path];

    if (isLoading) {
      return <Skeleton active paragraph={{ rows: 6 }} />;
    }
    if (!details) {
      return <div className="svl-save-details-empty">{t('app.saves.detailsLoadFailed')}</div>;
    }

    return (
      <div className="svl-save-details">
        <Row gutter={[16, 12]}>
          <Col span={8}>
            <Statistic
              title={t('app.saves.farmName')}
              value={save.farm_name || save.name}
              prefix={<HomeOutlined />}
              valueStyle={{ fontSize: 16 }}
            />
          </Col>
          <Col span={8}>
            <Statistic
              title={t('app.saves.currentDate')}
              value={`Y${save.year} ${save.current_season} D${save.day_of_month}`}
              prefix={<CalendarOutlined />}
              valueStyle={{ fontSize: 16 }}
            />
          </Col>
          <Col span={8}>
            <Statistic
              title={t('app.saves.playTime')}
              value={formatHours(save.hours_played)}
              prefix={<ClockCircleOutlined />}
              valueStyle={{ fontSize: 16 }}
            />
          </Col>
        </Row>

        <div className="svl-save-details-section">
          <div className="svl-save-details-title">
            <TrophyOutlined /> {t('app.saves.progressSection')}
          </div>
          <Row gutter={[16, 8]}>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.totalSkills')}:</span>
                <span className="svl-detail-value">{save.total_skill_levels}</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.spouse')}:</span>
                <span className="svl-detail-value">
                  {save.spouse || t('app.saves.single')}
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.mineLevel')}:</span>
                <span className="svl-detail-value">
                  {save.deepest_mine_level > 0 ? t('app.saves.floorN', { n: save.deepest_mine_level }) : '-'}
                </span>
              </div>
            </Col>
          </Row>
          <div className="svl-detail-achievements">
            {save.has_finished_community_center && (
              <Tag color="green" className="svl-achievement-tag">
                <HomeOutlined /> {t('app.saves.ccCompleted')}
              </Tag>
            )}
            {save.ginger_island_unlocked && (
              <Tag color="orange" className="svl-achievement-tag">
                <GlobalOutlined /> {t('app.saves.gingerIsland')}
              </Tag>
            )}
            {save.activated_golden_parrot && (
              <Tag color="gold" className="svl-achievement-tag">
                <CrownOutlined /> {t('app.saves.goldenParrot')}
              </Tag>
            )}
            {save.stardrops_found > 0 && (
              <Tag color="purple" className="svl-achievement-tag">
                <StarOutlined /> {t('app.saves.stardrops', { count: save.stardrops_found })}
              </Tag>
            )}
            {save.grandpa_score > 0 && (
              <Tag color="cyan" className="svl-achievement-tag">
                <GiftOutlined /> {t('app.saves.grandpaCandles', { count: save.grandpa_score })}
              </Tag>
            )}
          </div>
        </div>

        <Row gutter={[16, 12]}>
          <Col span={8}>
            <Statistic
              title={t('app.saves.statMoney')}
              value={details.money}
              suffix="g"
              prefix={<DollarOutlined />}
              valueStyle={{ color: '#faad14', fontSize: 18 }}
            />
          </Col>
          <Col span={8}>
            <Statistic
              title={t('app.saves.statTotalEarned')}
              value={details.total_money_earned}
              suffix="g"
              prefix={<TrophyOutlined />}
              valueStyle={{ color: '#52c41a', fontSize: 18 }}
            />
          </Col>
          <Col span={8}>
            <Statistic
              title={t('app.saves.statDaysPlayed')}
              value={details.days_played}
              suffix={t('app.saves.days')}
              prefix={<CalendarOutlined />}
              valueStyle={{ fontSize: 18 }}
            />
          </Col>
        </Row>

        <div className="svl-save-details-section">
          <div className="svl-save-details-title">
            <ThunderboltOutlined /> {t('app.saves.skillsSection')}
          </div>
          <Row gutter={[8, 8]}>
            {[
              { key: 'farming', label: t('app.saves.skillFarming'), level: details.farming_level, color: '#52c41a', icon: '🌾' },
              { key: 'mining', label: t('app.saves.skillMining'), level: details.mining_level, color: '#1890ff', icon: '⛏️' },
              { key: 'foraging', label: t('app.saves.skillForaging'), level: details.foraging_level, color: '#13c2c2', icon: '🌳' },
              { key: 'fishing', label: t('app.saves.skillFishing'), level: details.fishing_level, color: '#722ed1', icon: '🎣' },
              { key: 'combat', label: t('app.saves.skillCombat'), level: details.combat_level, color: '#f5222d', icon: '⚔️' },
            ].map((s) => (
              <Col span={12} key={s.key}>
                <div className="svl-skill-row">
                  <div className="svl-skill-label">
                    <span className="svl-skill-icon">{s.icon}</span>
                    <span>{s.label}</span>
                    <span className="svl-skill-level" style={{ color: s.color }}>Lv.{s.level}</span>
                  </div>
                  <Progress
                    percent={(s.level / 10) * 100}
                    showInfo={false}
                    strokeColor={s.color}
                    size="small"
                  />
                </div>
              </Col>
            ))}
          </Row>
        </div>

        <div className="svl-save-details-section">
          <div className="svl-save-details-title">
            <HeartOutlined /> {t('app.saves.socialSection')}
          </div>
          <Row gutter={[16, 8]}>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.spouse')}:</span>
                <span className="svl-detail-value">
                  {details.is_married ? (
                    <><HeartOutlined style={{ color: '#eb2f96' }} /> {details.spouse}</>
                  ) : (
                    t('app.saves.single')
                  )}
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.friendshipCount')}:</span>
                <span className="svl-detail-value">{details.friendship_count}</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.maxFriendship')}:</span>
                <span className="svl-detail-value">
                  {details.max_friendship_npc || '-'} ({details.max_friendship_points})
                </span>
              </div>
            </Col>
          </Row>
        </div>

        <div className="svl-save-details-section">
          <div className="svl-save-details-title">
            <FireOutlined /> {t('app.saves.worldSection')}
          </div>
          <Row gutter={[16, 8]}>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.mineLevel')}:</span>
                <span className="svl-detail-value">
                  {details.deepest_mine_level > 0 ? t('app.saves.floorN', { n: details.deepest_mine_level }) : '-'}
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.skullCavernLevel')}:</span>
                <span className="svl-detail-value">
                  {details.deepest_skull_cavern_level > 0 ? t('app.saves.floorN', { n: details.deepest_skull_cavern_level }) : '-'}
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.grandpaScore')}:</span>
                <span className="svl-detail-value">{details.grandpa_score}/4</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.stardropsFound')}:</span>
                <span className="svl-detail-value">
                  <StarOutlined style={{ color: '#faad14' }} /> {details.stardrops_found}/7
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.perfectionScore')}:</span>
                <span className="svl-detail-value">{details.perfection_score}%</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.perfectionWaivers')}:</span>
                <span className="svl-detail-value">{details.perfection_waivers}</span>
              </div>
            </Col>
          </Row>
          <div className="svl-detail-achievements">
            {details.has_finished_community_center ? (
              <Tag color="green" className="svl-achievement-tag">
                <HomeOutlined /> {t('app.saves.ccCompleted')}
              </Tag>
            ) : (
              <Tag className="svl-achievement-tag">
                <HomeOutlined /> {t('app.saves.ccNotCompleted')}
              </Tag>
            )}
            {details.has_joja_mart_run ? (
              <Tag color="blue" className="svl-achievement-tag">
                <ShoppingOutlined /> {t('app.saves.jojaCompleted')}
              </Tag>
            ) : null}
            {details.ginger_island_unlocked ? (
              <Tag color="orange" className="svl-achievement-tag">
                <GlobalOutlined /> {t('app.saves.gingerIsland')}
              </Tag>
            ) : (
              <Tag className="svl-achievement-tag">
                <GlobalOutlined /> {t('app.saves.gingerIslandLocked')}
              </Tag>
            )}
            {details.activated_golden_parrot ? (
              <Tag color="gold" className="svl-achievement-tag">
                <CrownOutlined /> {t('app.saves.goldenParrot')}
              </Tag>
            ) : null}
            {details.treasure_totems_used > 0 ? (
              <Tag color="cyan" className="svl-achievement-tag">
                <GiftOutlined /> {t('app.saves.treasureTotems', { count: details.treasure_totems_used })}
              </Tag>
            ) : null}
            {details.times_fed_raccoons > 0 ? (
              <Tag color="lime" className="svl-achievement-tag">
                🦝 {t('app.saves.raccoonsFed', { count: details.times_fed_raccoons })}
              </Tag>
            ) : null}
          </div>
        </div>

        <div className="svl-save-details-section">
          <div className="svl-save-details-title">
            <BookOutlined /> {t('app.saves.collectionsSection')}
          </div>
          <Row gutter={[16, 8]}>
            <Col span={6}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.buildingCount')}:</span>
                <span className="svl-detail-value">{details.building_count}</span>
              </div>
            </Col>
            <Col span={6}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.cabinCount')}:</span>
                <span className="svl-detail-value">{details.cabin_count}</span>
              </div>
            </Col>
            <Col span={6}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.itemCount')}:</span>
                <span className="svl-detail-value">{details.item_count}</span>
              </div>
            </Col>
            <Col span={6}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.questCount')}:</span>
                <span className="svl-detail-value">
                  {details.completed_quest_count}/{details.quest_count}
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.cookingRecipes')}:</span>
                <span className="svl-detail-value">{details.cooking_recipes}</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.craftingRecipes')}:</span>
                <span className="svl-detail-value">{details.crafting_recipes}</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.totalRecipes')}:</span>
                <span className="svl-detail-value">{details.recipes_known}</span>
              </div>
            </Col>
          </Row>
        </div>

        <div className="svl-save-details-section">
          <div className="svl-save-details-title">
            <ToolOutlined /> {t('app.saves.systemSection')}
          </div>
          <Row gutter={[16, 8]}>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.gameVersion')}:</span>
                <span className="svl-detail-value">{details.game_version}</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.health')}:</span>
                <span className="svl-detail-value">
                  <HeartOutlined style={{ color: '#f5222d' }} /> {details.health}/{details.max_health}
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.stamina')}:</span>
                <span className="svl-detail-value">
                  <ThunderboltOutlined style={{ color: '#52c41a' }} /> {details.stamina}/{details.max_stamina}
                </span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.timeOfDay')}:</span>
                <span className="svl-detail-value">{formatTimeOfDay(details.time_of_day)}</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.saveFileSize')}:</span>
                <span className="svl-detail-value">{(details.raw_xml_size / 1024).toFixed(1)} KB</span>
              </div>
            </Col>
            <Col span={8}>
              <div className="svl-detail-item">
                <span className="svl-detail-label">{t('app.saves.farmSize')}:</span>
                <span className="svl-detail-value">{(details.file_size_bytes / 1024 / 1024).toFixed(2)} MB</span>
              </div>
            </Col>
          </Row>
        </div>
      </div>
    );
  };

  const renderSaveCard = (save: SaveInfo) => {
    const panelKey = save.save_path;
    return (
      <Collapse
        key={panelKey}
        className="svl-save-collapse"
        activeKey={expandedKeys.filter((k) => k === panelKey)}
        onChange={(keys) => {
          const keyArr = Array.isArray(keys) ? keys : [keys];
          handleExpandChange(keyArr);
        }}
        expandIcon={({ isActive }) =>
          isActive ? <DownCircleOutlined /> : <UpCircleOutlined rotate={180} />
        }
        items={[
          {
            key: panelKey,
            showArrow: false,
            label: (
              <Card
                className="svl-save-card"
                title={renderSaveHeader(save)}
                extra={
                  <div className="svl-save-card-extra">
                    <Tooltip title={t('app.saves.lastModified')}>
                      <span className="svl-save-last-modified">
                        {save.last_modified}
                      </span>
                    </Tooltip>
                    <Button
                      size="small"
                      type="text"
                      icon={expandedKeys.includes(panelKey) ? <UpCircleOutlined /> : <DownCircleOutlined />}
                      onClick={(e) => {
                        e.stopPropagation();
                        const isExpanded = expandedKeys.includes(panelKey);
                        const next = isExpanded
                          ? expandedKeys.filter((k) => k !== panelKey)
                          : [...expandedKeys, panelKey];
                        handleExpandChange(next);
                      }}
                    >
                      {t('app.saves.details')}
                    </Button>
                  </div>
                }
              >
                {renderSaveSummary(save)}

                {requiredMods.length > 0 && (
                  <div className="svl-save-required-mods">
                    <Tooltip
                      title={
                        <div>
                          <div style={{ marginBottom: 4 }}>{t('app.saves.requiredMods')}:</div>
                          {requiredMods.map((mod) => (
                            <div key={mod.unique_id}>• {mod.name}</div>
                          ))}
                        </div>
                      }
                    >
                      <Tag color="orange" className="svl-required-mod-badge">
                        <LockOutlined style={{ marginRight: 4 }} />
                        {t('app.saves.requiredMods')} × {requiredMods.length}
                      </Tag>
                    </Tooltip>
                  </div>
                )}

                <div className="svl-save-profile-section">
                  <div className="svl-save-profile-selector">
                    <LinkOutlined style={{ color: 'var(--svl-text-muted)' }} />
                    <Select
                      size="small"
                      style={{ flex: 1, minWidth: 0 }}
                      value={getProfileSelectValue(save)}
                      onChange={(value) => handleProfileChange(save, value)}
                      loading={changingProfile === save.save_path}
                      placeholder={t('app.saves.bindProfile')}
                      options={[
                        { value: '__none__', label: t('app.saves.noBinding') },
                        ...profiles.map((p) => ({
                          value: p.name,
                          label: `${p.name} (${p.enabled_count}/${p.total_mods})`,
                        })),
                      ]}
                    />
                    {save.linked_profile && (
                      <Tooltip title={t('app.saves.unbind')}>
                        <Button
                          size="small"
                          type="text"
                          danger
                          icon={<DisconnectOutlined />}
                          onClick={() => handleUnlink(save)}
                        />
                      </Tooltip>
                    )}
                  </div>
                  {save.linked_profile && activeProfileName && save.linked_profile !== activeProfileName && (
                    <div className="svl-save-profile-status">
                      <Tag className="svl-tag-warning" style={{ marginLeft: 0, fontSize: 11 }}>
                        {t('app.saves.profileMismatch')}
                      </Tag>
                    </div>
                  )}
                </div>

                <div className="svl-save-actions">
                  <Button
                    size="small"
                    icon={<SaveOutlined />}
                    onClick={() => handleBackup(save)}
                  >
                    {t('app.saves.backup')}
                  </Button>
                  <Button
                    size="small"
                    icon={<UndoOutlined />}
                    onClick={() => handleRestore(save)}
                    disabled={save.backup_count === 0}
                  >
                    {t('app.saves.restore')}
                  </Button>
                  <Button
                    size="small"
                    type="primary"
                    icon={<PlayCircleOutlined />}
                    loading={launchingSave === save.save_path}
                    onClick={() => handleLaunchWithProfile(save)}
                    className="svl-save-launch-btn"
                  >
                    {t('app.saves.launchWithProfile')}
                  </Button>
                </div>
              </Card>
            ),
            children: renderDetails(save),
          },
        ]}
      />
    );
  };

  return (
    <div className="svl-content">
      <div className="svl-saves-page">
        <div className="svl-saves-header">
          <div className="svl-saves-title">
            <DatabaseOutlined /> {t('app.saves.title')}
          </div>
          <div className="svl-saves-actions">
            <Button
              icon={expandedKeys.length === saves.length && saves.length > 0 ? <UpCircleOutlined /> : <DownCircleOutlined />}
              onClick={handleToggleAll}
              disabled={saves.length === 0}
            >
              {expandedKeys.length === saves.length && saves.length > 0
                ? t('app.saves.collapseAll')
                : t('app.saves.expandAll')}
            </Button>
            <Button
              icon={<FolderOpenOutlined />}
              onClick={handleOpenSaveLocation}
            >
              {t('app.saves.openSaveLocation')}
            </Button>
            <Button
              icon={<ClockCircleOutlined />}
              onClick={loadSaves}
              loading={loading}
            >
              {t('app.saves.refresh')}
            </Button>
          </div>
        </div>

        {saves.length === 0 && !loading ? (
          <Empty description={t('app.saves.noSaves')} />
        ) : (
          <>
            <div className="svl-saves-grid">
              {saves
                .slice((currentPage - 1) * pageSize, currentPage * pageSize)
                .map((save) => renderSaveCard(save))}
            </div>
            {saves.length > pageSize && (
              <div className="svl-saves-pagination">
                <Pagination
                  current={currentPage}
                  pageSize={pageSize}
                  total={saves.length}
                  onChange={(page) => setCurrentPage(page)}
                  showSizeChanger={false}
                />
              </div>
            )}
          </>
        )}

        <Modal
          title={t('app.saves.backup')}
          open={showBackupModal}
          onOk={doBackup}
          onCancel={() => setShowBackupModal(false)}
          okText={t('app.saves.backup')}
          cancelText={t('app.common.cancel')}
          confirmLoading={backingUp}
        >
          <p>{t('app.saves.backupConfirm', { name: selectedSave?.name })}</p>
        </Modal>

        <Modal
          title={t('app.saves.restore')}
          open={showRestoreModal}
          onCancel={() => setShowRestoreModal(false)}
          footer={null}
          width={600}
        >
          <List
            dataSource={backups}
            renderItem={(backup) => (
              <List.Item
                actions={[
                  <Popconfirm
                    title={t('app.saves.restoreConfirm')}
                    onConfirm={() => doRestore(backup)}
                    okText={t('app.common.confirm')}
                    cancelText={t('app.common.cancel')}
                  >
                    <Button
                      size="small"
                      type="primary"
                      loading={restoring}
                    >
                      {t('app.saves.restore')}
                    </Button>
                  </Popconfirm>,
                ]}
              >
                <List.Item.Meta
                  title={backup.name}
                  description={`${backup.backup_time} · ${backup.size_mb.toFixed(2)} MB`}
                />
              </List.Item>
            )}
          />
        </Modal>

        <Modal
          title={selectedSave ? `${t('app.saves.detailsTitle')} - ${selectedSave.character_name}` : t('app.saves.detailsTitle')}
          open={showDetailsModal}
          onCancel={() => setShowDetailsModal(false)}
          footer={null}
          width={760}
        >
          {selectedSave && renderDetails(selectedSave)}
        </Modal>
      </div>
    </div>
  );
}
