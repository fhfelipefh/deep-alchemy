import { defineConfig } from 'vite';

export default defineConfig({
  base: './',
  server: {
    port: 3000,
    host: true, // Permite testar via rede local no celular Android diretamente
  },
  build: {
    target: 'es2022',
  },
});
