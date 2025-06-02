/*
this.env/index.js
ⓝⓔⓤⓡⓞⓝⓢ.ⓜⓔ
🆂🆄🅸🅶🅽                                                                                                            
--------------------------------
For more information, visit: https://neurons.me*/
import express from 'express';
import os from 'os';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const PORT = 31337;

const envDir = path.join(os.homedir(), '.thisenv');
const meFile = path.join(envDir, 'this.me.json');
const keysFile = path.join(envDir, 'keys.json');

if (!fs.existsSync(envDir)) fs.mkdirSync(envDir, { recursive: true });

// Simple endpoint to ping the environment
app.get('/ping', (_req, res) => {
  res.json({ status: 'this.env is alive', envDir });
});

// Endpoint to get this.me object
app.get('/me', (_req, res) => {
  if (fs.existsSync(meFile)) {
    res.json(JSON.parse(fs.readFileSync(meFile, 'utf-8')));
  } else {
    res.status(404).json({ error: 'this.me not found' });
  }
});

// Start the server
app.listen(PORT, () => {
  console.log(`this.env running on http://localhost:${PORT}`);
});
