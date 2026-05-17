function onMessage(event)
	require("lib")
	print("Hello from test/main.lua")
	print_r(event, "  ")
end

t.on("dummy.message", onMessage)
