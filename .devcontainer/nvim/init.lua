-- Torwell Neovim defaults for the devcontainer
vim.opt.number = true
vim.opt.relativenumber = true
vim.opt.expandtab = true
vim.opt.shiftwidth = 2
vim.opt.tabstop = 2
vim.opt.smartindent = true
vim.opt.termguicolors = true
vim.opt.updatetime = 250
vim.opt.signcolumn = "yes"
vim.opt.clipboard = "unnamedplus"
vim.g.mapleader = " "

local ensure = function(pkg)
  local status, _ = pcall(require, pkg)
  if not status then
    vim.cmd.packadd(pkg)
  end
end

ensure('plenary')

vim.api.nvim_create_autocmd('BufWritePre', {
  pattern = { '*.ts', '*.js', '*.json', '*.svelte', '*.rs', '*.md' },
  callback = function()
    vim.lsp.buf.format({ async = false })
  end,
})
