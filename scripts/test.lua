function onMessage(event)
	print("Hello from test.lua")
	print("will never be executed")
end
t.on("_undefined event_", onMessage)
