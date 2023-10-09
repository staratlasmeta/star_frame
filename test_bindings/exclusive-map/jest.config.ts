import { Config } from 'jest';

const config: Config = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testPathIgnorePatterns: ['<rootDir>/dist'],
  setupFilesAfterEnv: ['jest-expect-message'],
  testTimeout: 30000,
  openHandlesTimeout: 0,
};

export default config;
