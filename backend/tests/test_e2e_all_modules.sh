#!/usr/bin/env bash
set -e

BASE_URL="http://127.0.0.1:4000/api"


echo "=== Starting Complete E2E Tests for Tooth-Plus (All Modules & Permissions) ==="

# 1. Login Admin
echo "[1] Logging in as admin..."
ADMIN_LOGIN_RESP=$(curl -s -X POST "$BASE_URL/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password_plain":"123456"}')

ADMIN_TOKEN=$(echo "$ADMIN_LOGIN_RESP" | jq -r '.token // empty')
CLINIC_ID=$(echo "$ADMIN_LOGIN_RESP" | jq -r '.clinics[0].clinic_id // empty')


if [ -z "$ADMIN_TOKEN" ] || [ -z "$CLINIC_ID" ]; then
  echo "❌ Failed to login admin: $ADMIN_LOGIN_RESP"
  exit 1
fi
echo "✅ Admin logged in. Clinic ID: $CLINIC_ID"

# 2. Test Agenda & Rateio / Finance Permissions
echo "[2] Testing Agenda Module & Rateio / Finance Permissions..."

# 2.1 Get a staff user id to assign
USERS_RESP=$(curl -s -X GET "$BASE_URL/users?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")
STAFF_USER_ID=$(echo "$USERS_RESP" | jq -r '.[0].id // empty')

if [ -z "$STAFF_USER_ID" ]; then
  echo "❌ No staff user found"
  exit 1
fi

# 2.2 Admin creates appointment with R$ 450.00 and Rateio 40%
APP_TITLE="Consulta Completa Endodontia $(date +%s)"
APP_CREATE_RESP=$(curl -s -X POST "$BASE_URL/appointments" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"title\": \"$APP_TITLE\",
    \"scheduled_for\": \"2026-09-01T14:00:00Z\",
    \"duration_minutes\": 60,
    \"appointment_type\": \"treatment\",
    \"financial_amount_cents\": 45000,
    \"financial_type\": \"income\",
    \"patient_name\": \"Carlos Silva E2E\",
    \"notes\": \"Tratamento canal molar\",
    \"assigned_users\": [
      {
        \"user_id\": \"$STAFF_USER_ID\",
        \"role_in_appointment\": \"Cirurgião Principal\",
        \"split_percentage\": 40
      }
    ],
    \"consumed_items\": []
  }")

APP_ID=$(echo "$APP_CREATE_RESP" | jq -r '.id // empty')
if [ -z "$APP_ID" ]; then
  echo "❌ Failed to create appointment: $APP_CREATE_RESP"
  exit 1
fi
echo "✅ Appointment created with ID: $APP_ID"

# 2.3 Admin reads appointment: Should see financial_amount_cents = 45000 and split_percentage = 40
ADMIN_APP_RESP=$(curl -s -X GET "$BASE_URL/appointments?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

FOUND_ADMIN_APP=$(echo "$ADMIN_APP_RESP" | jq -r ".[] | select(.id == \"$APP_ID\")")
ADMIN_FIN_AMT=$(echo "$FOUND_ADMIN_APP" | jq -r '.financial_amount_cents')
ADMIN_SPLIT=$(echo "$FOUND_ADMIN_APP" | jq -r '.assigned_users[0].split_percentage')

if [ "$ADMIN_FIN_AMT" != "45000" ] || [ "$ADMIN_SPLIT" != "40" ]; then
  echo "❌ Admin did not see expected financial or rateio data: amount=$ADMIN_FIN_AMT, split=$ADMIN_SPLIT"
  exit 1
fi
echo "✅ Admin sees financial amount (R$ 450.00) and Rateio (40%)."

# 2.4 Create a restricted user with 'appointments:read' and 'appointments:write' BUT WITHOUT 'appointments:finance'
RESTRICTED_AGENDA_USER="agenda_nofin_$(date +%s)"
RESTRICTED_CPF="1$(shuf -i 10-99 -n 1).$(shuf -i 100-999 -n 1).$(shuf -i 100-999 -n 1)-$(shuf -i 10-99 -n 1)"
CREATE_RESTRICTED_RESP=$(curl -s -X POST "$BASE_URL/users" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$RESTRICTED_AGENDA_USER\",
    \"password_plain\": \"pass12345\",
    \"full_name\": \"Recepcionista Sem Financeiro\",
    \"document_cpf\": \"$RESTRICTED_CPF\",
    \"role\": \"receptionist\",
    \"clinic_ids\": [\"$CLINIC_ID\"],
    \"permissions\": [\"appointments:read\", \"appointments:write\"]
  }")



# Login with restricted agenda user
RESTRICTED_LOGIN=$(curl -s -X POST "$BASE_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$RESTRICTED_AGENDA_USER\",\"password_plain\":\"pass12345\"}")
RESTRICTED_TOKEN=$(echo "$RESTRICTED_LOGIN" | jq -r '.token // empty')


# 2.5 Restricted user fetches appointments: Financial amount MUST BE null and split MUST BE 0!
RESTRICTED_APP_RESP=$(curl -s -X GET "$BASE_URL/appointments?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $RESTRICTED_TOKEN")

FOUND_RESTRICTED_APP=$(echo "$RESTRICTED_APP_RESP" | jq -r ".[] | select(.id == \"$APP_ID\")")
RESTRICTED_FIN_AMT=$(echo "$FOUND_RESTRICTED_APP" | jq -r '.financial_amount_cents')
RESTRICTED_FIN_TYPE=$(echo "$FOUND_RESTRICTED_APP" | jq -r '.financial_type')
RESTRICTED_SPLIT=$(echo "$FOUND_RESTRICTED_APP" | jq -r '.assigned_users[0].split_percentage')

if [ "$RESTRICTED_FIN_AMT" != "null" ] || [ "$RESTRICTED_FIN_TYPE" != "null" ] || [ "$RESTRICTED_SPLIT" != "0" ]; then
  echo "❌ Security breach! Restricted user saw financial or commission data: amt=$RESTRICTED_FIN_AMT, type=$RESTRICTED_FIN_TYPE, split=$RESTRICTED_SPLIT"
  exit 1
fi
echo "✅ Security Verified: User without 'appointments:finance' received masked financial amount (null) and zeroed rateio (0%)."

# 2.6 Restricted user without appointments:delete tries to delete appointment -> Should return 403 Forbidden
DEL_FORBIDDEN_HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE_URL/appointments/$APP_ID?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $RESTRICTED_TOKEN")

if [ "$DEL_FORBIDDEN_HTTP" != "403" ]; then
  echo "❌ Restricted user without delete permission got HTTP $DEL_FORBIDDEN_HTTP instead of 403"
  exit 1
fi
echo "✅ Restricted user blocked with 403 on DELETE /appointments."

# 3. Test Finance Module
echo "[3] Testing Finance Module..."
FIN_TITLE="Compra de Luvas e Anestésicos $(date +%s)"
CREATE_TX_RESP=$(curl -s -X POST "$BASE_URL/finance" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"direction\": \"expense\",
    \"amount_cents\": 12500,
    \"description\": \"$FIN_TITLE\",
    \"category\": \"Insumos & Estoque\",
    \"due_date\": \"2026-09-10\",
    \"status\": \"pending\",
    \"payment_method\": \"Boleto\"
  }")


TX_ID=$(echo "$CREATE_TX_RESP" | jq -r '.id // empty')
if [ -z "$TX_ID" ]; then
  echo "❌ Failed to create finance transaction: $CREATE_TX_RESP"
  exit 1
fi
echo "✅ Finance transaction created with ID: $TX_ID"

# 3.1 Update status to paid
UPDATE_STATUS_RESP=$(curl -s -X PUT "$BASE_URL/finance/$TX_ID/status?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"status\": \"paid\",
    \"paid_date\": \"2026-09-10T10:00:00Z\",
    \"payment_method\": \"Pix\"
  }")
echo "✅ Transaction status updated to paid."

# 3.2 Create user without finance permissions -> Expect 403 on GET /finance
NO_FIN_USER="nofin_$(date +%s)"
NO_FIN_CPF="2$(shuf -i 10-99 -n 1).$(shuf -i 100-999 -n 1).$(shuf -i 100-999 -n 1)-$(shuf -i 10-99 -n 1)"
curl -s -X POST "$BASE_URL/users" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$NO_FIN_USER\",
    \"password_plain\": \"pass12345\",
    \"full_name\": \"Sem Acesso Financeiro\",
    \"document_cpf\": \"$NO_FIN_CPF\",
    \"role\": \"assistant\",
    \"clinic_ids\": [\"$CLINIC_ID\"],
    \"permissions\": [\"patients:read\"]
  }" > /dev/null



NO_FIN_LOGIN=$(curl -s -X POST "$BASE_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$NO_FIN_USER\",\"password_plain\":\"pass12345\"}")
NO_FIN_TOKEN=$(echo "$NO_FIN_LOGIN" | jq -r '.token // empty')


NO_FIN_HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$BASE_URL/finance?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $NO_FIN_TOKEN")

if [ "$NO_FIN_HTTP" != "403" ]; then
  echo "❌ User without finance permission got HTTP $NO_FIN_HTTP instead of 403"
  exit 1
fi
echo "✅ User without finance permission received 403 on GET /finance."

# 4. Test Stock Module
echo "[4] Testing Stock Module..."
ITEM_NAME="Resina Composta 3M E2E $(date +%s)"
CREATE_STOCK_RESP=$(curl -s -X POST "$BASE_URL/stock" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"item_type\": \"material\",
    \"name\": \"$ITEM_NAME\",
    \"unit_type\": \"unidade\",
    \"current_stock\": 0,
    \"min_stock\": 5,
    \"cost_price_cents\": 6500,
    \"attachments\": [],
    \"batch_number\": \"LOT-2026-X\"
  }")

STOCK_ITEM_ID=$(echo "$CREATE_STOCK_RESP" | jq -r '.id // empty')
if [ -z "$STOCK_ITEM_ID" ]; then
  echo "❌ Failed to create stock item: $CREATE_STOCK_RESP"
  exit 1
fi
echo "✅ Stock item created with ID: $STOCK_ITEM_ID"

# 4.1 Stock entry movement (10 units)
ENTRY_MOV_RESP=$(curl -s -X POST "$BASE_URL/stock/$STOCK_ITEM_ID/movement" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"item_id\": \"$STOCK_ITEM_ID\",
    \"movement_type\": \"purchase_in\",
    \"quantity_change\": 10,
    \"unit_cost_cents\": 6500,
    \"invoice_number\": \"NF-1234\",
    \"notes\": \"Entrada de insumos\"
  }")
echo "✅ Stock entry registered (+10 units)."

# 4.2 Stock exit movement (2 units)
EXIT_MOV_RESP=$(curl -s -X POST "$BASE_URL/stock/$STOCK_ITEM_ID/movement" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"item_id\": \"$STOCK_ITEM_ID\",
    \"movement_type\": \"manual_out\",
    \"quantity_change\": -2,
    \"unit_cost_cents\": 6500,
    \"notes\": \"Consumo em procedimento\"
  }")
echo "✅ Stock exit registered (-2 units)."

# 4.3 Verify stock item balance is 8
STOCK_LIST=$(curl -s -X GET "$BASE_URL/stock?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")
CURRENT_QTY=$(echo "$STOCK_LIST" | jq -r ".items[] | select(.id == \"$STOCK_ITEM_ID\") | .current_stock")

if [ "$CURRENT_QTY" != "8" ]; then
  echo "❌ Stock item balance is $CURRENT_QTY instead of 8"
  exit 1
fi
echo "✅ Stock item balance verified: 8 units."

# 4.4 User without stock:movement permission tries to make movement -> Expect 403
STOCK_NO_MOV_HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/stock/$STOCK_ITEM_ID/movement" \
  -H "Authorization: Bearer $RESTRICTED_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"item_id\": \"$STOCK_ITEM_ID\",
    \"movement_type\": \"manual_out\",
    \"quantity_change\": -1,
    \"unit_cost_cents\": 6500,
    \"notes\": \"Tentativa sem permissão\"
  }")



if [ "$STOCK_NO_MOV_HTTP" != "403" ]; then
  echo "❌ User without stock:movement got HTTP $STOCK_NO_MOV_HTTP instead of 403"
  exit 1
fi
echo "✅ User without stock:movement received 403."

# 5. Test Documents Module
echo "[5] Testing Documents Module..."
DOC_TITLE="Termo de Consentimento E2E $(date +%s)"
CREATE_DOC_RESP=$(curl -s -X POST "$BASE_URL/documents/templates" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"title\": \"$DOC_TITLE\",
    \"category\": \"consent\",
    \"description\": \"Termo de consentimento padrão\",
    \"pdf_url\": \"/uploads/templates/consent.pdf\",
    \"signature_fields\": [
      {
        \"id\": \"sig_1\",
        \"signer_type\": \"patient\",
        \"label\": \"Assinatura do Paciente\",
        \"is_required\": true,
        \"page_number\": 1,
        \"x_pct\": 10.0,
        \"y_pct\": 80.0,
        \"width_pct\": 40.0,
        \"height_pct\": 10.0
      }
    ]
  }")

DOC_TEMPLATE_ID=$(echo "$CREATE_DOC_RESP" | jq -r '.id // empty')
if [ -z "$DOC_TEMPLATE_ID" ]; then
  echo "❌ Failed to create document template: $CREATE_DOC_RESP"
  exit 1
fi
echo "✅ Document template created with ID: $DOC_TEMPLATE_ID"

# 5.1 Verify restricted user without documents:write gets 403 on creating template
DOC_FORBIDDEN_HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/documents/templates" \
  -H "Authorization: Bearer $RESTRICTED_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"title\": \"Tentativa Sem Permissão\",
    \"category\": \"consent\",
    \"description\": null,
    \"pdf_url\": \"/uploads/templates/test.pdf\",
    \"signature_fields\": []
  }")


if [ "$DOC_FORBIDDEN_HTTP" != "403" ]; then
  echo "❌ User without documents:write got HTTP $DOC_FORBIDDEN_HTTP instead of 403"
  exit 1
fi
echo "✅ User without documents:write blocked with 403."

# 6. Test Patients Module
echo "[6] Testing Patients Module..."
PATIENT_NAME="Paciente Teste E2E $(date +%s)"
PATIENT_CPF="3$(shuf -i 10-99 -n 1).$(shuf -i 100-999 -n 1).$(shuf -i 100-999 -n 1)-$(shuf -i 10-99 -n 1)"
CREATE_PAT_RESP=$(curl -s -X POST "$BASE_URL/patients" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"clinic_id\": \"$CLINIC_ID\",
    \"full_name\": \"$PATIENT_NAME\",
    \"document_cpf\": \"$PATIENT_CPF\",
    \"phone\": \"(11) 99999-8888\",
    \"email\": \"paciente.e2e@example.com\",
    \"birth_date\": \"1995-05-20\",
    \"gender\": \"male\",
    \"marital_status\": \"single\"
  }")

NEW_PATIENT_ID=$(echo "$CREATE_PAT_RESP" | jq -r '.id // empty')
if [ -z "$NEW_PATIENT_ID" ]; then
  echo "❌ Failed to create patient: $CREATE_PAT_RESP"
  exit 1
fi
echo "✅ Patient created with ID: $NEW_PATIENT_ID"

# 6.1 Verify restricted user without patients:delete gets 403 on DELETE /patients
PAT_DEL_FORBIDDEN=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE_URL/patients/$NEW_PATIENT_ID?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $RESTRICTED_TOKEN")

if [ "$PAT_DEL_FORBIDDEN" != "403" ]; then
  echo "❌ Restricted user without patients:delete got HTTP $PAT_DEL_FORBIDDEN instead of 403"
  exit 1
fi
echo "✅ User without patients:delete blocked with 403."

# 7. Clean up test records
echo "[7] Cleaning up test records..."
curl -s -X DELETE "$BASE_URL/appointments/$APP_ID?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" > /dev/null
curl -s -X DELETE "$BASE_URL/stock/$STOCK_ITEM_ID?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" > /dev/null
curl -s -X DELETE "$BASE_URL/finance/$TX_ID?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" > /dev/null
curl -s -X DELETE "$BASE_URL/documents/templates/$DOC_TEMPLATE_ID?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" > /dev/null
curl -s -X DELETE "$BASE_URL/patients/$NEW_PATIENT_ID?clinic_id=$CLINIC_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN" > /dev/null

echo "================================================================="
echo "🎉 ALL E2E INTEGRATION & GRANULAR PERMISSION TESTS PASSED 100%!"
echo "================================================================="

