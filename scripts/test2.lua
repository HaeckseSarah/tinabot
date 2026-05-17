function onMessage(event)
	print("Hello from test2.lua")
	p.dummy.ping(event.message)
end

t.on("dummy.message", onMessage)
