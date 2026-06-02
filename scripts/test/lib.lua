function print_r(t, indent)
	indent = indent or ""
	if type(t) == "table" then
		for k, v in pairs(t) do
			if type(v) == "table" then
				print(indent .. tostring(k) .. " => Table {")
				print_r(v, indent .. "  ")
				print(indent .. "}")
			else
				print(indent .. tostring(k) .. " => " .. tostring(v))
			end
		end
	else
		print(indent .. tostring(t))
	end
end
