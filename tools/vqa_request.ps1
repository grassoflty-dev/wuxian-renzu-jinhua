$apiKey = "sk_tr_4hgjNjV9IO5nF1xLU6RbdbUTPMZGlnBdLU3b7BoDYF4"
$baseUrl = "https://tokenrhythm.studio/v1/chat/completions"

$frontUrl = Get-Content "$env:TEMP\front_url.txt" -Raw
$faceUrl = Get-Content "$env:TEMP\face_url.txt" -Raw

# Also get reference illustration
$refPath = "C:\Users\PUBLIC_USER\Desktop\itwillclaude\games\wuxian-horror-ch1\tools\shots\pc_zhengzha.png"
if (Test-Path $refPath) {
    $refBytes = [System.IO.File]::ReadAllBytes($refPath)
    $refB64 = [Convert]::ToBase64String($refBytes, 0)
    $refUrl = "data:image/png;base64,$refB64"
} else {
    $refUrl = $null
}

$prompt = @"
你是视觉质检员，需要对《郑吒》体素模型的渲染质量进行详细评估。

## 图片说明
- **图1（正面全景）**: 体素模型正面全身照
- **图2（脸部特写）**: 体素模型面部近景特写  
- **图3（参考立绘）**: 原始立绘角色设定图

请参考立绘的特点：黑色短发、粗眉、深炭色T恤、战术长裤、肌肉型约7头身、宽肩窄腰

## 评估要求

### 对每张图逐一描述细节
分别对三张图片做详细描述：人物姿态、服装细节、表情特征、光影效果、体素风格呈现等。

### 五官细节验收（主要看 face 图 + front 图）:
1. **竖瞳** — 是否清晰可见？瞳孔形状如何？
2. **眉毛怒峰形态** — 是否有愤怒的山峰状挑角？还是平直的？
3. **挺鼻** — 鼻梁是否高挺？鼻尖形态？
4. **紧抿唇** — 嘴唇是否呈一条紧密的线条？还是张开的？
5. **灰短袖露出** — armSpan微张时是否能看见灰色短袖/内搭？
6. **肩甲回缩** — 肩部护甲是否收缩不突出？
7. **肱二头肌隆起** — 手臂是否有明显的肌肉鼓包？
8. **腰带扣** — 腰部能否辨认出扣环细节？
9. **大腿口袋** — 裤腿两侧是否有口袋突起？
10. **buzz蓬松发型** — 是否是短而蓬松的寸头发型？

### 配色和还原度
- T恤颜色是否为深炭色/黑色？接近立绘吗？
- 裤子色调是否正确（战术深色系）？
- 肤色大致是什么色调？
- 整体配色与立绘的匹配程度如何？

### 身体比例观察
- 头身比大约多少（目测头数量×身高）？
- 四肢长度是否协调？
- 肩膀宽度相对于身体是否偏宽（符合宽肩设定）？
- 腰部是否收紧（窄腰）？

请直接输出详细中文评估报告，格式如下：

## 正面全景图分析
[详细描述]

## 脸部特写分析  
[详细描述]

## 参考立绘对比
[对比描述]

## 细节验收清单
竖瞳: ✓/✗ - ...
眉毛怒峰: ✓/✗ - ...
...

## 配色评估
...

## 身体比例评估
...
"@

$body = @{
    model = "qwen3.7-flash"
    messages = @(
        @{
            role = "user"
            content = @(
                @{ type = "image_url"; image_url = @{ url = $frontUrl } },
                @{ type = "image_url"; image_url = @{ url = $faceUrl } },
                @{ type = "image_url"; image_url = @{ url = $refUrl } }
            )
        },
        @{
            role = "assistant"
            content = $prompt
        }
    )
    max_tokens = 8192
    temperature = 0.3
} | ConvertTo-Json -Depth 10

$headers = @{
    "Authorization" = "Bearer $apiKey"
    "Content-Type" = "application/json"
}

try {
    $response = Invoke-RestMethod -Uri $baseUrl -Method Post -Headers $headers -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) -ContentType "application/json"
    $content = $response.choices[0].message.content
    $content | Set-Content -Path "$env:TEMP\vqa_result.txt" -Encoding UTF8
    Write-Output "SUCCESS: Response saved, length: $($content.Length)"
} catch {
    Write-Output "ERROR: $_"
    if ($_.ErrorDetails) { Write-Output "Details: $($_.ErrorDetails.Message)" }
    throw $_
}
