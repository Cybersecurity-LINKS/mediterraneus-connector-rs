-- SPDX-FileCopyrightText: 2024 Fondazione LINKS

-- SPDX-License-Identifier: GPL-3.0-or-later

SELECT alias -- alias is unique, distinct is not strictly necessary 
FROM connector.assets, connector.identities
WHERE publisher = connector.identities.id AND eth_address = $1;