import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { message, Modal, Input, Tag, Button, Empty, Table, Checkbox, Spin, Divider, Tooltip } from 'antd';
import { open, save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import { listen } from '@tauri-apps/api/event';
import {
  PlusOutlined,
  FolderOutlined,
  CheckCircleOutlined,
  EditOutlined,
  ExportOutlined,
  ImportOutlined,
  CopyOutlined,
  SwapOutlined,
  LogoutOutlined,
  PictureOutlined,
  DownloadOutlined,
} from '@ant-design/icons';
import {
  profileCreate,
  profileList,
  profileGetActive,
  profileSwitch,
  profileDelete,
  updateProfileMods,
  profileGetModStates,
  profileClearActive,
  profileCopy,
  profileExport,
  profileImport,
  scanProfileMods,
  checkSmapiStatus,
  scanMods,
  type ProfileListItem,
  type ProfileModInfo,
  type SmapiInfo,
  type ModInfo,
} from '../utils/tauri-api';

export default function ProfilesPage() {
  const { t } = useTranslation();
  const [gamePath, setGamePath] = useState<string>('');
  const [profiles, setProfiles] = useState<ProfileListItem[]>([]);
  const [activeProfile, setActiveProfile] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [newProfileName, setNewProfileName] = useState('');
  const [createModIds, setCreateModIds] = useState<Set<string>>(new Set());
  const [createAllMods, setCreateAllMods] = useState<ProfileModInfo[]>([]);
  const [createLoading, setCreateLoading] = useState(false);
  const [creating, setCreating] = useState(false);

  const [editModalOpen, setEditModalOpen] = useState(false);
  const [editProfileName, setEditProfileName] = useState('');
  const [editModStates, setEditModStates] = useState<Record<string, boolean>>({});
  const [editAllMods, setEditAllMods] = useState<ProfileModInfo[]>([]);
  const [editLoading, setEditLoading] = useState(false);
  const [editSaving, setEditSaving] = useState(false);

  const [copyModalOpen, setCopyModalOpen] = useState(false);
  const [copyFromProfile, setCopyFromProfile] = useState('');
  const [copyNewName, setCopyNewName] = useState('');
  const [copying, setCopying] = useState(false);

  const [exiting, setExiting] = useState(false);

  const [cardModalOpen, setCardModalOpen] = useState(false);
  const [cardProfileMods, setCardProfileMods] = useState<ModInfo[]>([]);
  const [cardProfileName, setCardProfileName] = useState('');
  const [cardLoading, setCardLoading] = useState(false);

  useEffect(() => {
    checkSmapiStatus()
      .then((info: SmapiInfo) => {
        if (info.game_path) {
          setGamePath(info.game_path);
        }
      })
      .catch(() => {});
  }, []);

  const loadProfiles = useCallback(async () => {
    if (!gamePath) return;
    setLoading(true);
    try {
      const list = await profileList(gamePath);
      setProfiles([...list]);
      const active = await profileGetActive(gamePath);
      setActiveProfile(active);
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.loadFailed'));
    } finally {
      setLoading(false);
    }
  }, [gamePath, t]);

  useEffect(() => {
    if (gamePath) {
      loadProfiles();
    }
  }, [gamePath, loadProfiles]);

  useEffect(() => {
    if (!gamePath) return;
    let unlisten: (() => void) | null = null;
    listen('profile-changed', () => {
      loadProfiles();
    }).then(fn => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }, [gamePath, loadProfiles]);

  useEffect(() => {
    if (!gamePath) return;
    let unlisten: (() => void) | null = null;
    listen('mod-install-progress', (event: any) => {
      if (event.payload?.step === 'completed' || event.payload?.step === 'done') {
        loadProfiles();
      }
    }).then(fn => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }, [gamePath, loadProfiles]);

  const handleOpenCreateModal = async () => {
    setCreateModalOpen(true);
    setCreateLoading(true);
    try {
      const mods = await scanProfileMods(gamePath);
      setCreateAllMods(mods);
      setCreateModIds(new Set(mods.map((m: ProfileModInfo) => m.unique_id)));
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.loadModStatesFailed'));
    } finally {
      setCreateLoading(false);
    }
  };

  const handleCloseCreateModal = () => {
    setCreateModalOpen(false);
    setNewProfileName('');
    setCreateModIds(new Set());
    setCreateAllMods([]);
  };

  const handleToggleCreateMod = (uniqueId: string) => {
    const mod = createAllMods.find(m => m.unique_id === uniqueId);
    if (mod?.is_required) return;
    setCreateModIds(prev => {
      const next = new Set(prev);
      if (next.has(uniqueId)) {
        next.delete(uniqueId);
      } else {
        next.add(uniqueId);
      }
      return next;
    });
  };

  const handleCreateSelectAll = () => {
    setCreateModIds(new Set(createAllMods.map(m => m.unique_id)));
  };

  const handleCreateDeselectAll = () => {
    const requiredIds = createAllMods.filter(m => m.is_required).map(m => m.unique_id);
    setCreateModIds(new Set(requiredIds));
  };

  const handleCreate = async () => {
    if (!newProfileName.trim()) {
      message.warning(t('app.profilesPage.nameRequired'));
      return;
    }
    setCreating(true);
    try {
      await profileCreate(gamePath, newProfileName.trim(), Array.from(createModIds));
      message.success(t('app.profilesPage.createSuccess'));
      handleCloseCreateModal();
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.createFailed'));
    } finally {
      setCreating(false);
    }
  };

  const handleSwitch = async (profileName: string) => {
    try {
      await profileSwitch(gamePath, profileName);
      message.success(t('app.profilesPage.switchSuccess', { name: profileName }));
      setActiveProfile(profileName);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.switchFailed'));
    }
  };

  const handleDelete = async (profileName: string) => {
    try {
      await profileDelete(gamePath, profileName);
      message.success(t('app.profilesPage.deleteSuccess'));
      if (activeProfile === profileName) {
        setActiveProfile(null);
      }
      await loadProfiles();
    } catch (err: any) {
      const detail = err?.toString() || '';
      message.error(`${t('app.profilesPage.deleteFailed')}${detail ? ': ' + detail : ''}`);
    }
  };

  const handleExitProfile = async () => {
    if (!activeProfile) return;
    setExiting(true);
    try {
      await profileClearActive(gamePath);
      message.success(t('app.profilesPage.exitSuccess'));
      setActiveProfile(null);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.exitFailed'));
    } finally {
      setExiting(false);
    }
  };

  const handleOpenEditModal = async (profileName: string) => {
    setEditProfileName(profileName);
    setEditModalOpen(true);
    setEditLoading(true);
    try {
      const [mods, states] = await Promise.all([
        scanProfileMods(gamePath),
        profileGetModStates(gamePath, profileName),
      ]);
      setEditAllMods(mods);
      setEditModStates({ ...states });
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.loadModStatesFailed'));
    } finally {
      setEditLoading(false);
    }
  };

  const handleOpenCardModal = async (profileName: string) => {
    setCardProfileName(profileName);
    setCardModalOpen(true);
    setCardLoading(true);
    try {
      const allMods = await scanMods(gamePath);
      const states = await profileGetModStates(gamePath, profileName);
      const enabledMods = allMods.filter((m) => states[m.unique_id] !== false);
      setCardProfileMods(enabledMods);
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.loadModsFailed'));
    } finally {
      setCardLoading(false);
    }
  };

  const handleSaveCardImage = async () => {
    // Load real sprite images
    const loadImg = (src: string): Promise<HTMLImageElement> =>
      new Promise((resolve) => {
        const img = new Image();
        img.crossOrigin = 'anonymous';
        img.onload = () => resolve(img);
        img.onerror = () => resolve(img);
        img.src = src;
      });

    const [chickenImg, woodFenceImg] = await Promise.all([
      loadImg('/chicken.png'),
      loadImg('/wood-fence.png'),
    ]);

    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const cardWidth = 900;
    const signBorder = 36;
    const signInnerMargin = 24;
    const titleHeight = 72;
    const tableHeaderHeight = 44;
    const footerHeight = 64;
    const rowHeight = 40;
    const borderWidth = 4;

    const signContentWidth = cardWidth - signBorder * 2;
    const tableTopOffset = signBorder + signInnerMargin + titleHeight + tableHeaderHeight + 12;
    const totalHeight = tableTopOffset + cardProfileMods.length * rowHeight + footerHeight + signInnerMargin;

    canvas.width = cardWidth;
    canvas.height = totalHeight;

    const colors = {
      sky: '#87CEEB',
      skyWhite: '#C5E8F7',
      grass: '#567D46',
      grassLight: '#6B9B57',
      grassDark: '#3E5C2A',
      path: '#B8956A',
      pathLight: '#C9A87E',
      wood: '#8B5E3C',
      woodLight: '#A0724A',
      woodDark: '#5E3D1F',
      parchment: '#F5DEB3',
      parchmentDark: '#E8CFA0',
      text: '#3E2723',
      textLight: '#5D4037',
      gold: '#FFD700',
      vine: '#2E7D32',
      vineLight: '#4CAF50',
      leaf: '#388E3C',
    };

    // Load farmer sprite from user file
    const farmerSprite = await loadImg('/farmer-sprite.png');

    // Background: Use real farm screenshot (cover mode to avoid stretch)
    const bgImg = await loadImg('/images/stardew-farm-screenshot.jpg');
    const bgScale = Math.max(cardWidth / bgImg.width, totalHeight / bgImg.height);
    const bgDrawW = bgImg.width * bgScale;
    const bgDrawH = bgImg.height * bgScale;
    const bgX = (cardWidth - bgDrawW) / 2;
    const bgY = (totalHeight - bgDrawH) / 2;
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(bgImg, bgX, bgY, bgDrawW, bgDrawH);
    ctx.imageSmoothingEnabled = true;

    // Darken background slightly for readability
    ctx.fillStyle = 'rgba(0,0,0,0.15)';
    ctx.fillRect(0, 0, cardWidth, totalHeight);

    const signX = signBorder;
    const signY = signBorder;
    const signW = signContentWidth;
    const signH = totalHeight - signBorder * 2;

    ctx.fillStyle = 'rgba(0,0,0,0.15)';
    roundRect(ctx, signX + 6, signY + 6, signW, signH, 12);
    ctx.fill();

    ctx.fillStyle = colors.wood;
    roundRect(ctx, signX, signY, signW, signH, 12);
    ctx.fill();

    ctx.strokeStyle = colors.woodLight;
    ctx.lineWidth = 3;
    roundRect(ctx, signX, signY, signW, signH, 12);
    ctx.stroke();

    ctx.strokeStyle = colors.woodDark;
    ctx.lineWidth = borderWidth;
    roundRect(ctx, signX + 4, signY + 4, signW - 8, signH - 8, 10);
    ctx.stroke();

    const innerX = signX + 4 + borderWidth + 4;
    const innerY = signY + 4 + borderWidth + 4;
    const innerW = signW - 8 - borderWidth * 2 - 8;
    const innerH = signH - 8 - borderWidth * 2 - 8;

    const parchmentGrad = ctx.createLinearGradient(innerX, innerY, innerX, innerY + innerH);
    parchmentGrad.addColorStop(0, colors.parchment);
    parchmentGrad.addColorStop(1, colors.parchmentDark);
    ctx.fillStyle = parchmentGrad;
    roundRect(ctx, innerX, innerY, innerW, innerH, 6);
    ctx.fill();

    ctx.strokeStyle = colors.woodLight;
    ctx.lineWidth = 2;
    roundRect(ctx, innerX, innerY, innerW, innerH, 6);
    ctx.stroke();

    const titleY = innerY + 12;
    ctx.fillStyle = colors.woodLight;
    for (let dx = innerX + innerW / 2 - 60; dx < innerX + innerW / 2 + 60; dx += 6) {
      ctx.fillRect(dx, titleY + titleHeight - 8, 4, 4);
    }

    ctx.fillStyle = colors.text;
    ctx.font = 'bold 32px "SimHei", "Microsoft YaHei", sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(t('app.profilesPage.cardTitle'), cardWidth / 2, titleY + titleHeight - 20);

    const drawLeaf = (lx: number, ly: number, angle: number) => {
      ctx.save();
      ctx.translate(lx, ly);
      ctx.rotate(angle);
      ctx.imageSmoothingEnabled = false;
      const fw = woodFenceImg.width;
      const fh = woodFenceImg.height;
      ctx.drawImage(woodFenceImg, 0, 0, fw, fh, -12, -8, 24, 16);
      ctx.imageSmoothingEnabled = true;
      ctx.restore();
    };
    drawLeaf(innerX + 20, titleY + titleHeight / 2, -0.3);
    drawLeaf(innerX + 40, titleY + titleHeight / 2 - 10, -0.1);
    drawLeaf(innerX + innerW - 20, titleY + titleHeight / 2, 0.3);
    drawLeaf(innerX + innerW - 40, titleY + titleHeight / 2 - 10, 0.1);

    const drawChicken = (cx: number, cy: number, size: number = 48) => {
      ctx.imageSmoothingEnabled = false;
      const chW = chickenImg.width;
      const chH = chickenImg.height;
      const chScale = size / Math.max(chW, chH);
      const chDrawW = chW * chScale;
      const chDrawH = chH * chScale;
      ctx.drawImage(chickenImg, 0, 0, chW, chH, cx - chDrawW / 2, cy - chDrawH / 2, chDrawW, chDrawH);
      ctx.imageSmoothingEnabled = true;
    };
    drawChicken(innerX + innerW - 30, titleY + titleHeight / 2);

    const tableTop = innerY + titleHeight + 16;
    const tableLeft = innerX + 16;
    const tableRight = innerX + innerW - 16;
    const tableWidth = tableRight - tableLeft;
    const nameColW = tableWidth * 0.45;
    const idColW = tableWidth * 0.55;

    ctx.fillStyle = 'rgba(139, 120, 90, 0.15)';
    ctx.fillRect(tableLeft, tableTop, tableWidth, tableHeaderHeight);

    ctx.strokeStyle = colors.woodLight;
    ctx.lineWidth = 1;
    ctx.strokeRect(tableLeft, tableTop, tableWidth, tableHeaderHeight);

    ctx.fillStyle = colors.text;
    ctx.font = 'bold 16px "SimHei", "Microsoft YaHei", sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('模组名称', tableLeft + nameColW / 2, tableTop + tableHeaderHeight / 2 + 6);
    ctx.fillText('模组尾号', tableLeft + nameColW + idColW / 2, tableTop + tableHeaderHeight / 2 + 6);

    ctx.strokeStyle = colors.woodLight;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(tableLeft, tableTop + tableHeaderHeight);
    ctx.lineTo(tableRight, tableTop + tableHeaderHeight);
    ctx.stroke();

    const iconColors = ['#FF69B4', '#4CAF50', '#2196F3', '#FF9800', '#9C27B0', '#F44336', '#795548'];
    cardProfileMods.forEach((mod, idx) => {
      const rowY = tableTop + tableHeaderHeight + idx * rowHeight;

      if (idx % 2 === 0) {
        ctx.fillStyle = 'rgba(139, 120, 90, 0.08)';
        ctx.fillRect(tableLeft, rowY, tableWidth, rowHeight);
      }

      ctx.strokeStyle = 'rgba(139, 120, 90, 0.3)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(tableLeft, rowY + rowHeight);
      ctx.lineTo(tableRight, rowY + rowHeight);
      ctx.stroke();

      const iconColor = iconColors[idx % iconColors.length];
      const iconX = tableLeft + 10;
      const iconY = rowY + rowHeight / 2;

      ctx.fillStyle = iconColor;
      ctx.fillRect(iconX - 6, iconY - 6, 12, 12);
      ctx.fillStyle = 'rgba(255,255,255,0.3)';
      ctx.fillRect(iconX - 4, iconY - 4, 8, 8);

      ctx.fillStyle = colors.text;
      ctx.font = 'bold 14px "SimHei", "Microsoft YaHei", sans-serif';
      ctx.textAlign = 'left';
      let modName = mod.name;
      if (modName.length > 28) modName = modName.substring(0, 26) + '...';
      ctx.fillText(modName, tableLeft + 22, rowY + rowHeight / 2 + 5);

      const nexusDisplay = mod.nexus_mod_id ? String(mod.nexus_mod_id) : '-';
      ctx.fillStyle = colors.text;
      ctx.font = 'bold 14px "SimHei", "Microsoft YaHei", sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(nexusDisplay, tableLeft + nameColW + idColW / 2, rowY + rowHeight / 2 + 5);
    });

    const tableBottom = tableTop + tableHeaderHeight + cardProfileMods.length * rowHeight;
    ctx.strokeStyle = colors.woodLight;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(tableLeft, tableBottom);
    ctx.lineTo(tableRight, tableBottom);
    ctx.stroke();

    const footerTop = tableBottom + 20;
    ctx.strokeStyle = colors.woodLight;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(innerX + 40, footerTop);
    ctx.lineTo(innerX + innerW - 40, footerTop);
    ctx.stroke();

    ctx.fillStyle = colors.woodLight;
    for (let dx = innerX + innerW / 2 - 50; dx < innerX + innerW / 2 + 50; dx += 6) {
      ctx.fillRect(dx, footerTop + 4, 4, 4);
    }

    const drawFarmer = (fx: number, fy: number) => {
      ctx.imageSmoothingEnabled = false;
      const fsW = farmerSprite.width;
      const fsH = farmerSprite.height;
      const fsScale = 40 / Math.max(fsW, fsH);
      const fsDrawW = fsW * fsScale;
      const fsDrawH = fsH * fsScale;
      ctx.drawImage(farmerSprite, fx - fsDrawW / 2, fy - fsDrawH / 2, fsDrawW, fsDrawH);
      ctx.imageSmoothingEnabled = true;
    };
    drawFarmer(cardWidth / 2 - 100, footerTop + 16);
    drawChicken(cardWidth / 2 + 100, footerTop + 16);

    const drawVine = (side: 'left' | 'right', startY: number, endY: number) => {
      const baseX = side === 'left' ? signX + 12 : signX + signW - 12;
      const dir = side === 'left' ? 1 : -1;
      for (let vy = startY; vy < endY; vy += 24) {
        ctx.fillStyle = colors.vine;
        ctx.fillRect(baseX - 2, vy, 4, 24);
        ctx.fillStyle = colors.vineLight;
        ctx.beginPath();
        ctx.ellipse(baseX + dir * 10, vy + 6, 10, 5, dir * 0.5, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = colors.leaf;
        ctx.beginPath();
        ctx.ellipse(baseX + dir * 10, vy + 18, 8, 4, dir * 0.3, 0, Math.PI * 2);
        ctx.fill();
      }
    };
    drawVine('left', innerY + 20, innerY + innerH - 20);
    drawVine('right', innerY + 20, innerY + innerH - 20);

    const drawScrew = (sx: number, sy: number) => {
      ctx.fillStyle = '#888';
      ctx.beginPath();
      ctx.arc(sx, sy, 8, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = '#666';
      ctx.beginPath();
      ctx.arc(sx, sy, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = '#444';
      ctx.fillRect(sx - 3, sy - 1, 6, 2);
    };
    drawScrew(signX + 14, signY + 14);
    drawScrew(signX + signW - 14, signY + 14);
    drawScrew(signX + 14, signY + signH - 14);
    drawScrew(signX + signW - 14, signY + signH - 14);

    const dataUrl = canvas.toDataURL('image/png');
    const base64 = dataUrl.replace('data:image/png;base64,', '');
    const fileName = cardProfileName + '_模组清单.png';
    try {
      const filePath = await save({ filters: [{ name: 'PNG Image', extensions: ['png'] }], defaultPath: fileName });
      if (filePath) {
        const binaryString = atob(base64);
        const len = binaryString.length;
        const bytes = new Uint8Array(len);
        for (let i = 0; i < len; i++) bytes[i] = binaryString.charCodeAt(i);
        await writeFile(filePath, bytes);
        message.success(t('app.profilesPage.cardSaved', { path: filePath }));
      } else {
        message.info(t('app.profilesPage.saveCancelled'));
      }
    } catch (err: any) {
      message.error(t('app.profilesPage.saveFailed', { error: err?.toString() || t('app.common.unknownError') }));
    }
  };

  function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
    ctx.beginPath();
    ctx.moveTo(x + r, y);
    ctx.lineTo(x + w - r, y);
    ctx.quadraticCurveTo(x + w, y, x + w, y + r);
    ctx.lineTo(x + w, y + h - r);
    ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    ctx.lineTo(x + r, y + h);
    ctx.quadraticCurveTo(x, y + h, x, y + h - r);
    ctx.lineTo(x, y + r);
    ctx.quadraticCurveTo(x, y, x + r, y);
    ctx.closePath();
  }

  const handleToggleEditMod = (uniqueId: string) => {
    setEditModStates(prev => ({
      ...prev,
      [uniqueId]: !(prev[uniqueId] ?? false),
    }));
  };

  const handleEditSelectAll = () => {
    const next: Record<string, boolean> = {};
    editAllMods.forEach(m => { next[m.unique_id] = true; });
    setEditModStates(next);
  };

  const handleEditDeselectAll = () => {
    const next: Record<string, boolean> = {};
    editAllMods.forEach(m => { next[m.unique_id] = false; });
    setEditModStates(next);
  };

  const handleSaveEdit = async () => {
    setEditSaving(true);
    try {
      const enabledIds = Object.entries(editModStates)
        .filter(([_, enabled]) => enabled)
        .map(([id]) => id);
      await updateProfileMods(gamePath, editProfileName, enabledIds);
      message.success(t('app.profilesPage.saveModStatesSuccess'));
      setEditModalOpen(false);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.saveModStatesFailed'));
    } finally {
      setEditSaving(false);
    }
  };

  const handleOpenCopyModal = (profileName: string) => {
    setCopyFromProfile(profileName);
    setCopyNewName(`${profileName} (copy)`);
    setCopyModalOpen(true);
  };

  const handleCopy = async () => {
    if (!copyNewName.trim()) {
      message.warning(t('app.profilesPage.nameRequired'));
      return;
    }
    setCopying(true);
    try {
      await profileCopy(gamePath, copyFromProfile, copyNewName.trim());
      message.success(t('app.profilesPage.createSuccess'));
      setCopyModalOpen(false);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.createFailed'));
    } finally {
      setCopying(false);
    }
  };

  const handleExport = async (profileName: string) => {
    try {
      const selected = await save({
        title: t('app.profiles.exportProfile'),
        defaultPath: `${profileName}.svl_profile`,
        filters: [{ name: t('app.profileFile'), extensions: ['svl_profile', 'json'] }],
      });
      if (selected) {
        await profileExport(gamePath, profileName, selected as string);
        message.success(t('app.profiles.exportSuccess'));
      }
    } catch (err: any) {
      message.error(err?.toString() || t('app.profiles.exportFailed'));
    }
  };

  const handleImport = async () => {
    try {
      const selected = await open({
        title: t('app.profiles.importProfile'),
        filters: [{ name: t('app.profileFile'), extensions: ['svl_profile', 'json'] }],
        multiple: false,
      });
      if (selected) {
        await profileImport(gamePath, selected as string);
        message.success(t('app.profiles.importSuccess'));
        await loadProfiles();
      }
    } catch (err: any) {
      message.error(err?.toString() || t('app.profiles.importFailed'));
    }
  };

  const columns = [
    {
      title: t('app.profilesPage.profileName'),
      dataIndex: 'name',
      key: 'name',
      render: (name: string, record: ProfileListItem) => (
        <span style={{ fontWeight: 600, color: 'var(--svl-text-primary)' }}>
          {name}
          {record.is_active && (
            <Tag icon={<CheckCircleOutlined />} color="var(--svl-primary)" style={{ marginLeft: 8 }}>
              {t('app.profilesPage.active')}
            </Tag>
          )}
          {record.is_protected && (
            <Tag className="svl-tag-warning" style={{ marginLeft: 4 }}>
              {t('app.profilesPage.protected')}
            </Tag>
          )}
        </span>
      ),
    },
    {
      title: t('app.profilesPage.modCount'),
      key: 'mod_count',
      width: 120,
      render: (_: any, record: ProfileListItem) => (
        <span>{record.enabled_count}/{record.total_mods}</span>
      ),
    },
    {
      title: t('app.profilesPage.createdAt'),
      dataIndex: 'created_at',
      key: 'created_at',
      width: 180,
      render: (date: string) => {
        try {
          return <span style={{ color: 'var(--svl-text-muted)', fontSize: 13 }}>{new Date(date).toLocaleString()}</span>;
        } catch {
          return <span style={{ color: 'var(--svl-text-muted)', fontSize: 13 }}>{date}</span>;
        }
      },
    },
    {
      title: t('app.profilesPage.actions'),
      key: 'actions',
      width: 320,
      render: (_: any, record: ProfileListItem) => (
        <div style={{ display: 'flex', alignItems: 'center', gap: 0, flexWrap: 'wrap' }}>
          {!record.is_active ? (
            <Tooltip title={t('app.profiles.switch')}>
              <Button
                size="small"
                type="link"
                icon={<SwapOutlined />}
                onClick={() => handleSwitch(record.name)}
              >
                {t('app.profiles.switch')}
              </Button>
            </Tooltip>
          ) : null}
          <Button
            size="small"
            type="link"
            icon={<EditOutlined />}
            onClick={() => handleOpenEditModal(record.name)}
          >
            {t('app.profiles.editMods')}
          </Button>
          <Button
            size="small"
            type="link"
            icon={<PictureOutlined />}
            onClick={() => handleOpenCardModal(record.name)}
          >
            {t('app.profilesPage.generateCard')}
          </Button>
          <Divider type="vertical" />
          <Tooltip title={t('app.profilesPage.copy')}>
            <Button
              size="small"
              type="link"
              icon={<CopyOutlined />}
              onClick={() => handleOpenCopyModal(record.name)}
            />
          </Tooltip>
          <Tooltip title={t('app.profiles.exportProfile')}>
            <Button
              size="small"
              type="link"
              icon={<ExportOutlined />}
              onClick={() => handleExport(record.name)}
            />
          </Tooltip>
          {!record.is_protected && (
            <>
              <Divider type="vertical" />
              <Button
                size="small"
                type="link"
                danger
                onClick={() => {
                  Modal.confirm({
                    title: t('app.profilesPage.confirm'),
                    content: t('app.profiles.deleteConfirmName', { name: record.name }),
                    okText: t('app.profilesPage.confirm'),
                    cancelText: t('app.profilesPage.cancel'),
                    okButtonProps: { danger: true },
                    onOk: () => handleDelete(record.name),
                  });
                }}
              >
                {t('app.profilesPage.delete')}
              </Button>
            </>
          )}
        </div>
      ),
    },
  ];

  if (!gamePath) {
    return (
      <div className="svl-content">
        <div className="svl-profiles-page-card">
          <Empty description={t('app.profilesPage.noGamePath')} />
        </div>
      </div>
    );
  }

  return (
    <div className="svl-content">
      <div className="svl-profiles-page-card">
        <div className="svl-profiles-page-header">
          <div className="svl-profiles-page-icon">
            <FolderOutlined />
          </div>
          <div className="svl-profiles-page-title">
            {t('app.profiles.title')}
          </div>
          <div className="svl-profiles-page-actions">
            {activeProfile && (
              <>
                <Tag color="var(--svl-primary)" style={{ marginRight: 4 }}>
                  {t('app.profiles.activeProfile', { name: activeProfile })}
                </Tag>
                <Button
                  size="small"
                  icon={<LogoutOutlined />}
                  loading={exiting}
                  onClick={handleExitProfile}
                  style={{ marginRight: 8 }}
                >
                  {t('app.profiles.exitProfile')}
                </Button>
              </>
            )}
            <Button
              icon={<ImportOutlined />}
              onClick={handleImport}
              style={{ marginRight: 8 }}
            >
              {t('app.profiles.importProfile')}
            </Button>
            <Button
              className="svl-create-profile-btn"
              icon={<PlusOutlined />}
              onClick={handleOpenCreateModal}
            >
              {t('app.profiles.createNew')}
            </Button>
          </div>
        </div>

        <div className="svl-profiles-page-body">
          {loading && profiles.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '60px 0' }}>
              <Spin size="large" />
            </div>
          ) : profiles.length === 0 ? (
            <Empty description={t('app.profiles.empty')} />
          ) : (
            <Table
              dataSource={profiles}
              columns={columns}
              rowKey="name"
              pagination={false}
              loading={loading}
              size="middle"
              className="svl-profiles-table"
            />
          )}
        </div>
      </div>

      <Modal
        title={t('app.profiles.createNew')}
        open={createModalOpen}
        onCancel={handleCloseCreateModal}
        width={680}
        footer={[
          <Button key="cancel" onClick={handleCloseCreateModal}>
            {t('app.profilesPage.cancel')}
          </Button>,
          <Button key="create" type="primary" onClick={handleCreate} loading={creating}>
            {t('app.profilesPage.create')}
          </Button>,
        ]}
      >
        {createLoading ? (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Spin size="large" />
          </div>
        ) : (
          <div style={{ marginTop: 16, display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
              <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
                {t('app.profilesPage.profileName')}
              </div>
              <Input
                value={newProfileName}
                onChange={(e) => setNewProfileName(e.target.value)}
                placeholder={t('app.profilesPage.namePlaceholder')}
              />
            </div>
            <div>
              <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
                {t('app.profiles.selectMods')}
              </div>
              <div style={{ marginBottom: 8, display: 'flex', gap: 8 }}>
                <Button size="small" onClick={handleCreateSelectAll}>
                  {t('app.profilesPage.selectAll')}
                </Button>
                <Button size="small" onClick={handleCreateDeselectAll}>
                  {t('app.profilesPage.deselectAll')}
                </Button>
                <span style={{ marginLeft: 'auto', color: 'var(--svl-text-muted)', fontSize: 13, lineHeight: '32px' }}>
                  {t('app.profiles.totalSelected', { count: createModIds.size, total: createAllMods.length })}
                </span>
              </div>
              <div style={{ maxHeight: 300, overflow: 'auto', border: '1px solid var(--svl-border)', borderRadius: 8, padding: 8 }}>
                {createAllMods.length === 0 ? (
                  <Empty description={t('app.profilesPage.noModsFound')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
                ) : (
                  createAllMods.map((mod, idx) => (
                    <div
                      key={`${mod.unique_id}_${idx}`}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        padding: '5px 8px',
                        borderBottom: '1px solid var(--svl-border)',
                        borderRadius: 4,
                        opacity: mod.is_required ? 0.85 : 1,
                      }}
                    >
                      <Checkbox
                        checked={createModIds.has(mod.unique_id)}
                        onChange={() => handleToggleCreateMod(mod.unique_id)}
                        disabled={mod.is_required}
                      />
                      <span style={{ marginLeft: 8, flex: 1 }}>
                        {mod.name}
                        {mod.is_required && (
                          <Tag className="svl-tag-warning" style={{ marginLeft: 6, fontSize: 11, lineHeight: '18px' }}>
                            {t('app.modCard.requiredMod')}
                          </Tag>
                        )}
                      </span>
                      <span style={{ color: 'var(--svl-text-muted)', fontSize: 12 }}>{mod.version}</span>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        )}
      </Modal>

      <Modal
        title={t('app.profiles.editMods') + ' - ' + editProfileName}
        open={editModalOpen}
        onCancel={() => setEditModalOpen(false)}
        width={680}
        footer={[
          <Button key="cancel" onClick={() => setEditModalOpen(false)}>
            {t('app.profilesPage.cancel')}
          </Button>,
          <Button key="save" type="primary" onClick={handleSaveEdit} loading={editSaving}>
            {t('app.profilesPage.save')}
          </Button>,
        ]}
      >
        {editLoading ? (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Spin size="large" />
          </div>
        ) : (
          <div>
            <div style={{ marginBottom: 12, display: 'flex', gap: 8 }}>
              <Button size="small" onClick={handleEditSelectAll}>
                {t('app.profilesPage.selectAll')}
              </Button>
              <Button size="small" onClick={handleEditDeselectAll}>
                {t('app.profilesPage.deselectAll')}
              </Button>
              <span style={{ marginLeft: 'auto', color: 'var(--svl-text-muted)', fontSize: 13, lineHeight: '32px' }}>
                {t('app.profiles.totalSelected', { count: Object.values(editModStates).filter(Boolean).length, total: editAllMods.length })}
              </span>
            </div>
            <div style={{ maxHeight: 400, overflow: 'auto', border: '1px solid var(--svl-border)', borderRadius: 8, padding: 8 }}>
              {editAllMods.length === 0 ? (
                <Empty description={t('app.profilesPage.noModsFound')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
              ) : (
                editAllMods.map((mod, idx) => (
                  <div
                    key={`${mod.unique_id}_${idx}`}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      padding: '5px 8px',
                      borderBottom: '1px solid var(--svl-border)',
                      borderRadius: 4,
                    }}
                  >
                    <Checkbox
                      checked={editModStates[mod.unique_id] ?? false}
                      onChange={() => handleToggleEditMod(mod.unique_id)}
                      disabled={mod.is_required}
                    />
                    <span style={{ marginLeft: 8, flex: 1 }}>
                      {mod.name}
                      {mod.is_required && (
                        <Tag className="svl-tag-warning" style={{ marginLeft: 6, fontSize: 11, lineHeight: '18px' }}>
                          {t('app.modCard.requiredMod')}
                        </Tag>
                      )}
                    </span>
                    <span style={{ color: 'var(--svl-text-muted)', fontSize: 12 }}>{mod.version}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        )}
      </Modal>

      <Modal
        title={t('app.profilesPage.copyProfile')}
        open={copyModalOpen}
        onCancel={() => setCopyModalOpen(false)}
        onOk={handleCopy}
        confirmLoading={copying}
        okText={t('app.profilesPage.create')}
        cancelText={t('app.profilesPage.cancel')}
      >
        <div style={{ marginTop: 16 }}>
          <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
            {t('app.profilesPage.copyFrom')}: <strong>{copyFromProfile}</strong>
          </div>
          <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
            {t('app.profilesPage.profileName')}
          </div>
          <Input
            value={copyNewName}
            onChange={(e) => setCopyNewName(e.target.value)}
            placeholder={t('app.profilesPage.namePlaceholder')}
          />
        </div>
      </Modal>

      <Modal
        title={t('app.profilesPage.modCard')}
        open={cardModalOpen}
        onCancel={() => setCardModalOpen(false)}
        width={600}
        footer={[
          <Button key="close" onClick={() => setCardModalOpen(false)}>
            {t('app.profilesPage.close')}
          </Button>,
          <Button
            key="save"
            type="primary"
            icon={<DownloadOutlined />}
            onClick={handleSaveCardImage}
            disabled={cardProfileMods.length === 0}
          >
            {t('app.profilesPage.saveCardImage')}
          </Button>,
        ]}
      >
        {cardLoading ? (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Spin size="large" />
          </div>
        ) : (
          <div
            style={{
              marginTop: 16,
              borderRadius: 16,
              overflow: 'hidden',
              fontFamily: "'Microsoft YaHei', sans-serif",
              position: 'relative',
              boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
              backgroundImage: 'url(/images/stardew-farm-screenshot.jpg)',
              backgroundSize: 'cover',
              backgroundPosition: 'center',
              padding: 36,
            }}
          >
            {/* Wood sign */}
            <div
              style={{
                position: 'relative',
                zIndex: 10,
                background: '#8B5E3C',
                borderRadius: 12,
                padding: 4,
                boxShadow: '0 6px 24px rgba(0,0,0,0.2)',
              }}
            >
              {/* Screws */}
              <div style={{ position: 'absolute', top: 10, left: 10, width: 12, height: 12, borderRadius: '50%', background: '#666', border: '2px solid #888' }} />
              <div style={{ position: 'absolute', top: 10, right: 10, width: 12, height: 12, borderRadius: '50%', background: '#666', border: '2px solid #888' }} />
              <div style={{ position: 'absolute', bottom: 10, left: 10, width: 12, height: 12, borderRadius: '50%', background: '#666', border: '2px solid #888' }} />
              <div style={{ position: 'absolute', bottom: 10, right: 10, width: 12, height: 12, borderRadius: '50%', background: '#666', border: '2px solid #888' }} />

              {/* Inner parchment */}
              <div
                style={{
                  background: 'linear-gradient(180deg, #F5DEB3 0%, #E8CFA0 100%)',
                  borderRadius: 8,
                  padding: '16px 20px',
                  border: '4px solid #5E3D1F',
                }}
              >
                {/* Title */}
                <div style={{ textAlign: 'center', marginBottom: 16, position: 'relative' }}>
                  <img src="/wood-fence.png" alt="" style={{ width: 18, height: 18, verticalAlign: 'middle', marginRight: 4, imageRendering: 'pixelated' }} />
                  <span style={{ color: '#3E2723', fontSize: 28, fontWeight: 'bold' }}>我的星露谷模组清单</span>
                  <img src="/wood-fence.png" alt="" style={{ width: 18, height: 18, verticalAlign: 'middle', marginLeft: 4, imageRendering: 'pixelated' }} />
                  <img src="/chicken.png" alt="" style={{ position: 'absolute', right: 10, top: 2, width: 28, height: 28, imageRendering: 'pixelated' }} />
                </div>

                {/* Divider dots */}
                <div style={{ textAlign: 'center', color: '#A0724A', letterSpacing: 4, marginBottom: 12 }}>·····················</div>

                {/* Table header */}
                <div
                  style={{
                    display: 'flex',
                    background: 'rgba(139, 120, 90, 0.15)',
                    border: '1px solid #A0724A',
                    borderRadius: 4,
                    marginBottom: 4,
                    padding: '8px 12px',
                  }}
                >
                  <span style={{ flex: 1, color: '#3E2723', fontWeight: 'bold', fontSize: 15, textAlign: 'center' }}>模组名称</span>
                  <span style={{ flex: 1, color: '#3E2723', fontWeight: 'bold', fontSize: 15, textAlign: 'center' }}>模组尾号</span>
                </div>

                {/* Mod list */}
                <div style={{ maxHeight: 360, overflow: 'auto' }}>
                  {cardProfileMods.map((mod, idx) => {
                    const iconColors = ['#FF69B4', '#4CAF50', '#2196F3', '#FF9800', '#9C27B0', '#F44336', '#795548'];
                    const iconColor = iconColors[idx % iconColors.length];
                    return (
                      <div
                        key={idx}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          padding: '8px 12px',
                          borderBottom: '1px solid rgba(139, 120, 90, 0.3)',
                          background: idx % 2 === 0 ? 'rgba(139, 120, 90, 0.08)' : 'transparent',
                        }}
                      >
                        {/* Mod icon */}
                        <span
                          style={{
                            display: 'inline-block',
                            width: 14,
                            height: 14,
                            background: iconColor,
                            borderRadius: 3,
                            marginRight: 10,
                            position: 'relative',
                          }}
                        >
                          <span style={{ position: 'absolute', top: 2, left: 2, width: 10, height: 10, background: 'rgba(255,255,255,0.3)', borderRadius: 2 }} />
                        </span>
                        <span
                          style={{
                            flex: 1,
                            color: '#3E2723',
                            fontSize: 14,
                            fontWeight: 'bold',
                            whiteSpace: 'nowrap',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                          }}
                        >
                          {mod.name}
                        </span>
                        <span
                          style={{
                            flex: 1,
                            color: '#3E2723',
                            fontSize: 14,
                            fontWeight: 'bold',
                            textAlign: 'center',
                            whiteSpace: 'nowrap',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                          }}
                        >
                          {mod.nexus_mod_id ? String(mod.nexus_mod_id) : '-'}
                        </span>
                      </div>
                    );
                  })}
                </div>

                {/* Bottom divider */}
                <div style={{ textAlign: 'center', color: '#A0724A', letterSpacing: 4, margin: '12px 0 8px' }}>·····················</div>

                {/* Farmer and chicken footer */}
                <div style={{ textAlign: 'center' }}>
                  <img src="/farmer-sprite.png" alt="" style={{ width: 24, height: 32, verticalAlign: 'middle', marginRight: 8, imageRendering: 'pixelated' }} />
                  <img src="/chicken.png" alt="" style={{ width: 20, height: 20, verticalAlign: 'middle', marginLeft: 8, imageRendering: 'pixelated' }} />
                </div>
              </div>
            </div>

            {/* Vine decorations on sides */}
            <div style={{ position: 'absolute', top: 40, left: 8, opacity: 0.8, zIndex: 5 }}>
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', marginBottom: 16, imageRendering: 'pixelated' }} />
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', marginBottom: 16, imageRendering: 'pixelated' }} />
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', marginBottom: 16, imageRendering: 'pixelated' }} />
            </div>
            <div style={{ position: 'absolute', top: 40, right: 8, opacity: 0.8, zIndex: 5 }}>
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', marginBottom: 16, imageRendering: 'pixelated' }} />
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', marginBottom: 16, imageRendering: 'pixelated' }} />
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', marginBottom: 16, imageRendering: 'pixelated' }} />
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', marginBottom: 16, imageRendering: 'pixelated' }} />
              <img src="/wood-fence.png" alt="" style={{ width: 16, height: 16, display: 'block', imageRendering: 'pixelated' }} />
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
