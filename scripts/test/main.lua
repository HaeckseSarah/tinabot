function onMessage(event)
	require("lib")

	a = p.twitch.get_channel_information(string.sub(event.message, 4))
	print("Infos:")
	print_r(a)
end

t.on("dummy.message", {
	queue = "seq",
	filters = { { "message", "starts_with", "c? " } },
}, onMessage)
