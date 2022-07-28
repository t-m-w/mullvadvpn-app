import { ipcMain } from 'electron';

import { createIpcMain } from '../shared/ipc-helpers';
import { ipcSchema } from '../shared/ipc-schema';
import { DaemonRpc } from './daemon-rpc';

export const IpcMainEventChannel = createIpcMain(ipcSchema, ipcMain, DaemonRpc.getInstance());
