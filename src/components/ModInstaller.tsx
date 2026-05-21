import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { message, Modal, List, Tag, Button } from 'antd';
import { PlusOutlined, ExclamationCircleOutlined } from '@ant-design/icons';
import { open } from '@tauri-apps/plugin-dialog';
import { installMod, checkModDependencies, type ModDependencyCheck } from '../utils/tauri-api';

interface ModInstallerProps {
  modsPath: string;
  onInstallSuccess: () => void;
}

export default function ModInstaller({ modsPath, onInstallSuccess }: ModInstallerProps) {
  const { t } = useTranslation();
  const [installing, setInstalling] = useState(false);

  const handleInstallMod = async () => {
    try {
      const selected = await open({
        title: t('app.modInstaller.selectTitle'),
        filters: [{ name: 'MOD 压缩包', extensions: ['zip', '7z'] }],
        multiple: false,
      });

      if (!selected) return;

      const filePath = selected as string;

      setInstalling(true);

      const depCheck: ModDependencyCheck = await checkModDependencies(filePath, modsPath);

      if (depCheck.missing_dependencies.length > 0) {
        const requiredDeps = depCheck.missing_dependencies.filter(d => d.is_required);

        Modal.confirm({
          title: t('app.modInstaller.depCheckTitle'),
          icon: <ExclamationCircleOutlined />,
          content: (
            <div>
              <p>{t('app.modInstaller.depCheckDesc', { name: depCheck.mod_name })}</p>
              {requiredDeps.length > 0 && (
                <div style={{ marginBottom: 12 }}>
                  <Tag className="svl-tag-error">{t('app.modInstaller.requiredDeps')}</Tag>
                  <List
                    size="small"
                    dataSource={requiredDeps}
                    renderItem={(dep) => (
                      <List.Item>
                        <span style={{ color: 'var(--svl-error)' }}>{dep.unique_id}</span>
                        {dep.minimum_version && (
                          <Tag style={{ marginLeft: 8 }}>v{dep.minimum_version}+</Tag>
                        )}
                      </List.Item>
                    )}
                  />
                </div>
              )}
            </div>
          ),
          okText: requiredDeps.length > 0
            ? t('app.modInstaller.installAnyway')
            : t('app.modInstaller.continueInstall'),
          cancelText: t('app.modInstaller.cancel'),
          onOk: async () => {
            await doInstall(filePath);
          },
          onCancel: () => {
            setInstalling(false);
          },
        });
      } else {
        await doInstall(filePath);
      }
    } catch (err: any) {
      message.error(err?.toString() || t('app.modInstaller.installFailed'));
    } finally {
      if (!installing) setInstalling(false);
    }
  };

  const doInstall = async (filePath: string) => {
    try {
      const result = await installMod(filePath, modsPath);
      if (result.success) {
        message.success(result.message);
        onInstallSuccess();
      } else {
        message.error(result.message);
      }
    } catch (err: any) {
      message.error(err?.toString() || t('app.modInstaller.installFailed'));
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div className="svl-mod-installer">
      <Button
        className="svl-install-mod-btn"
        icon={<PlusOutlined />}
        onClick={handleInstallMod}
        loading={installing}
      >
        {installing ? t('app.modInstaller.installing') : t('app.modInstaller.installMod')}
      </Button>
    </div>
  );
}
