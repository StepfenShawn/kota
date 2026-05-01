kota.setup({
  model = "deepseek-flash",
  api_key = os.getenv("API_KEY"),
  api_base = "https://api.deepseek.com/v1",
  temperature = 1.0,
  
  tools = {
    enabled = { "grep_find", "write_file", "read_file" },
    disabled = { "delete_file" },
  },
  
  -- custom_tools = {
  --   my_tool = require("plugins.my_tool"),
  -- },
  
  commands = {
    ["fix"] = "analyze and fix the current file",
    ["test"] = "run tests for current file",
  },
  
  hooks = {
    before_execute = function(tool, args)
      -- ...
    end,
    after_execute = function(result)
      -- ...
    end,
  },
})