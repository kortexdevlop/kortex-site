import test from 'node:test';
import assert from 'node:assert/strict';
import {desktopRelease,installScript} from '../src/lib/desktop-release.ts';
const script=installScript();
test('install script verifies the published hash before running the installer',()=>{assert.ok(script.includes(desktopRelease.url));assert.ok(script.includes(desktopRelease.sha256));assert.ok(script.indexOf('Get-FileHash')<script.indexOf('Start-Process -FilePath $installer'))});
test('install script is silent and never closes the caller session',()=>{assert.ok(script.includes("-ArgumentList '/S'"));assert.doesNotMatch(script,/\bexit\b/)});
test('release filename stays in sync with the download URL',()=>{assert.ok(desktopRelease.url.endsWith('/'+desktopRelease.filename));assert.ok(desktopRelease.filename.includes(desktopRelease.version))});
